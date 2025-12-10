/*!
Build executor with parallel compilation
*/

use crate::build::{BuildCache, BuildConfig, BuildGraph};
use crate::common::Result;
use rayon::prelude::*;
use std::path::Path;

/// Build result
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// Number of modules compiled
    pub modules_compiled: usize,
    /// Number of modules cached
    pub modules_cached: usize,
    /// Build duration in milliseconds
    pub duration_ms: u128,
    /// Success status
    pub success: bool,
}

impl BuildResult {
    /// Create successful build result
    pub fn success(compiled: usize, cached: usize, duration: u128) -> Self {
        Self {
            modules_compiled: compiled,
            modules_cached: cached,
            duration_ms: duration,
            success: true,
        }
    }

    /// Create failed build result
    pub fn failed(duration: u128) -> Self {
        Self {
            modules_compiled: 0,
            modules_cached: 0,
            duration_ms: duration,
            success: false,
        }
    }
}

/// Build executor
pub struct BuildExecutor {
    config: BuildConfig,
    graph: BuildGraph,
    cache: BuildCache,
}

impl BuildExecutor {
    /// Create a new build executor
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            graph: BuildGraph::new(),
            cache: BuildCache::new(),
        }
    }

    /// Execute build
    pub fn execute(&self) -> Result<BuildResult> {
        let start = std::time::Instant::now();

        // Encontrar archivos .vela en el proyecto
        let vela_files = self.find_vela_files()?;
        if vela_files.is_empty() {
            println!("⚠️  No .vela files found in project");
            return Ok(BuildResult::success(0, 0, start.elapsed().as_millis()));
        }

        println!("📁 Found {} Vela files to compile", vela_files.len());

        // Construir grafo de dependencias desde archivos
        let mut temp_executor = BuildExecutor::new(self.config.clone());
        for file_path in &vela_files {
            temp_executor.graph_mut().add_module(file_path.clone());
        }

        // Obtener niveles de compilación (orden topológico)
        let levels = match temp_executor.graph.topological_sort() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("❌ Build failed: {}", e);
                return Ok(BuildResult::failed(start.elapsed().as_millis()));
            }
        };

        let mut compiled = 0;
        let mut cached = 0;

        // Compilar cada nivel en paralelo
        for level in levels {
            let results: Vec<Result<_>> = level
                .par_iter()
                .map(|&module_id| {
                    // Crear un executor separado para cada módulo para evitar problemas de mutabilidad
                    let mut module_executor = BuildExecutor::new(self.config.clone());
                    *module_executor.graph_mut() = temp_executor.graph.clone();

                    if let Some(module) = module_executor.graph.get_module(module_id) {
                        // Verificar cache
                        if self.config.incremental && self.cache.is_valid(&module.path).unwrap_or(false) {
                            return Ok((false, true)); // No compilado, pero en cache
                        }

                        // Compilar módulo
                        module_executor.compile_module(module_id)?;
                        Ok((true, false)) // Compilado
                    } else {
                        Ok((false, false))
                    }
                })
                .collect();

            // Procesar resultados
            for result in results {
                match result {
                    Ok((was_compiled, was_cached)) => {
                        if was_compiled {
                            compiled += 1;
                        }
                        if was_cached {
                            cached += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Compilation error: {}", e);
                        return Ok(BuildResult::failed(start.elapsed().as_millis()));
                    }
                }
            }
        }

        let duration = start.elapsed().as_millis();
        Ok(BuildResult::success(compiled, cached, duration))
    }

    /// Compile a single module
    fn compile_module(&mut self, module_id: crate::build::ModuleId) -> Result<()> {
        use std::path::Path;
        use vela_compiler::Compiler;
        use vela_compiler::config::Config;

        // Obtener información del módulo
        let module = self.graph.get_module(module_id)
            .ok_or_else(|| crate::common::Error::BuildFailed { message: format!("Module {} not found", module_id.0) })?;

        let module_name = module.path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        println!("🔨 Compiling module: {} ({})", module_name, module.path.display());

        // Verificar cache para compilación incremental
        if self.config.incremental && self.cache.is_valid(&module.path).unwrap_or(false) {
            println!("⚡ Module {} is up to date", module_name);
            return Ok(());
        }

        // Compilar usando el compilador Vela
        let mut compiler = Compiler::new(Config::default());
        let bytecode = compiler.compile_file(&module.path)
            .map_err(|e| crate::common::Error::BuildFailed { message: format!("Compilation failed: {:?}", e) })?;

        if compiler.has_errors() {
            let diagnostics = compiler.diagnostics();
            return Err(crate::common::Error::BuildFailed { message: format!("Compilation errors: {:?}", diagnostics) });
        }

        // Generar ruta de salida
        let output_path = self.get_output_path(&module.path);

        // Crear directorio de salida si no existe
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::common::Error::Io(e))?;
        }

        // Escribir bytecode
        let bytecode_len = bytecode.len();
        std::fs::write(&output_path, bytecode)
            .map_err(|e| crate::common::Error::Io(e))?;

        // Actualizar cache
        self.cache.store(module.path.clone(), Vec::new())
            .map_err(|e| crate::common::Error::BuildFailed { message: format!("Failed to update cache: {}", e) })?;

        println!("✅ Compiled {} -> {} ({} bytes)", module_name, output_path.display(), bytecode_len);

        Ok(())
    }

    /// Obtener ruta de salida para un archivo fuente
    fn get_output_path(&self, source_path: &Path) -> std::path::PathBuf {
        let relative_path = source_path.strip_prefix(&self.config.project_root)
            .unwrap_or(source_path);

        let mut output_path = self.config.output_dir.clone();
        output_path.push(relative_path);
        output_path.set_extension("velac"); // Extensión para bytecode compilado

        output_path
    }

    /// Encontrar todos los archivos .vela en el proyecto
    fn find_vela_files(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut vela_files = Vec::new();

        // Directorios a buscar (src/, examples/, packages/)
        let search_dirs = ["src", "examples", "packages"];

        for dir_name in &search_dirs {
            let dir_path = self.config.project_root.join(dir_name);
            if dir_path.exists() {
                self.collect_vela_files_recursive(&dir_path, &mut vela_files)?;
            }
        }

        // También buscar en la raíz del proyecto
        self.collect_vela_files_recursive(&self.config.project_root, &mut vela_files)?;

        Ok(vela_files)
    }

    /// Recolectar archivos .vela recursivamente
    fn collect_vela_files_recursive(&self, dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)
            .map_err(|e| crate::common::Error::Io(e))? {

            let entry = entry.map_err(|e| crate::common::Error::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                // Saltar directorios de build y dependencias
                if !self.should_skip_directory(&path) {
                    self.collect_vela_files_recursive(&path, files)?;
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("vela") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Verificar si un directorio debe ser saltado durante la búsqueda
    fn should_skip_directory(&self, path: &Path) -> bool {
        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        matches!(dir_name, "target" | "node_modules" | ".git" | "dist" | "build")
    }

    /// Get mutable reference to graph
    pub fn graph_mut(&mut self) -> &mut BuildGraph {
        &mut self.graph
    }

    /// Get mutable reference to cache
    pub fn cache_mut(&mut self) -> &mut BuildCache {
        &mut self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_result_success() {
        let result = BuildResult::success(10, 5, 100);
        assert!(result.success);
        assert_eq!(result.modules_compiled, 10);
        assert_eq!(result.modules_cached, 5);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_build_result_failed() {
        let result = BuildResult::failed(50);
        assert!(!result.success);
        assert_eq!(result.modules_compiled, 0);
    }

    #[test]
    fn test_new_executor() {
        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        assert_eq!(executor.graph.modules().count(), 0);
    }

    #[test]
    fn test_execute_empty_graph() {
        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        let result = executor.execute().unwrap();
        assert!(result.success);
        assert_eq!(result.modules_compiled, 0);
    }

    #[test]
    fn test_execute_single_module() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        executor.graph_mut().add_module(PathBuf::from("main.vela"));

        let result = executor.execute().unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_execute_with_dependencies() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        let graph = executor.graph_mut();
        let a = graph.add_module(PathBuf::from("a.vela"));
        let b = graph.add_module(PathBuf::from("b.vela"));
        graph.add_dependency(b, a);

        let result = executor.execute().unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_graph_mut() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        executor.graph_mut().add_module(PathBuf::from("test.vela"));
        assert_eq!(executor.graph.modules().count(), 1);
    }

    #[test]
    fn test_cache_mut() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        executor.cache_mut().clear();
        assert!(executor.cache.is_empty());
    }
}
