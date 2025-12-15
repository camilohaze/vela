/*!
Build executor with parallel compilation
*/

use crate::build::{BuildCache, BuildConfig, BuildGraph};
use crate::common::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

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
        let mut result = BuildResult::success(compiled, cached, duration);

        // Post-processing para targets específicos
        if let Some(target) = &self.config.target {
            match target.as_str() {
                "ios" => {
                    println!("📱 Generating iOS build artifacts...");
                    if let Err(e) = self.generate_ios_artifacts() {
                        eprintln!("❌ iOS build failed: {}", e);
                        return Ok(BuildResult::failed(duration));
                    }
                    println!("✅ iOS build artifacts generated successfully");
                }
                "android" => {
                    println!("🤖 Generating Android build artifacts...");
                    // TODO: Implement Android build
                    println!("⚠️  Android target not yet implemented");
                }
                "web" => {
                    println!("🌐 Generating Web build artifacts...");
                    // TODO: Implement Web build
                    println!("⚠️  Web target not yet implemented");
                }
                "desktop" => {
                    println!("🖥️  Generating Desktop build artifacts...");
                    // TODO: Implement Desktop build
                    println!("⚠️  Desktop target not yet implemented");
                }
                _ => {
                    println!("⚠️  Unknown target '{}', skipping post-processing", target);
                }
            }
        }

        Ok(result)
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

        // Solo buscar en src/ para evitar archivos de ejemplo con caracteres Unicode
        let src_dir = self.config.project_root.join("src");
        if src_dir.exists() {
            self.collect_vela_files_recursive(&src_dir, &mut vela_files)?;
        }

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

        matches!(dir_name, "target" | "node_modules" | ".git" | "dist" | "build" | "vm" | "examples" | "tests" | "tooling" | "docs" | ".github")
    }

    /// Get mutable reference to graph
    pub fn graph_mut(&mut self) -> &mut BuildGraph {
        &mut self.graph
    }

    /// Get mutable reference to cache
    pub fn cache_mut(&mut self) -> &mut BuildCache {
        &mut self.cache
    }

    /// Generate iOS-specific build artifacts
    pub fn generate_ios_artifacts(&self) -> Result<()> {
        use std::fs;
        use std::path::Path;

        // Crear directorio de output para iOS
        let ios_output_dir = self.config.output_dir.join("ios");
        fs::create_dir_all(&ios_output_dir)?;

        // Generar Package.swift para Swift Package Manager
        self.generate_package_swift(&ios_output_dir)?;

        // Generar código Swift/Objective-C wrapper
        self.generate_ios_wrapper(&ios_output_dir)?;

        // Generar AppDelegate y ViewController base
        self.generate_ios_app_structure(&ios_output_dir)?;

        // Copiar bytecode compilado
        self.copy_compiled_bytecode(&ios_output_dir)?;

        println!("📱 iOS artifacts generated in: {}", ios_output_dir.display());
        Ok(())
    }

    /// Generate Package.swift for Swift Package Manager
    pub fn generate_package_swift(&self, output_dir: &Path) -> Result<()> {
        let package_swift = r#"// swift-tools-version:5.7
import PackageDescription

let package = Package(
    name: "VelaApp",
    platforms: [
        .iOS(.v14)
    ],
    products: [
        .executable(name: "VelaApp", targets: ["VelaApp"])
    ],
    dependencies: [
        // Add any external dependencies here
    ],
    targets: [
        .executableTarget(
            name: "VelaApp",
            dependencies: [],
            path: "Sources"
        )
    ]
)
"#;

        let package_path = output_dir.join("Package.swift");
        std::fs::write(package_path, package_swift)?;
        println!("📄 Generated Package.swift");
        Ok(())
    }

    /// Generate iOS wrapper code
    fn generate_ios_wrapper(&self, output_dir: &Path) -> Result<()> {
        let sources_dir = output_dir.join("Sources");
        std::fs::create_dir_all(&sources_dir)?;

        // Generar main.swift
        let main_swift = r#"import Foundation
import UIKit

// Import Vela runtime (would be linked)
// extern func vela_ios_create_runtime() -> UnsafeMutableRawPointer?

@main
struct VelaApp {
    static func main() {
        UIApplicationMain(
            CommandLine.argc,
            CommandLine.unsafeArgv,
            nil,
            NSStringFromClass(AppDelegate.self)
        )
    }
}

class AppDelegate: UIResponder, UIApplicationDelegate {
    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        window = UIWindow(frame: UIScreen.main.bounds)
        window?.rootViewController = VelaViewController()
        window?.makeKeyAndVisible()
        return true
    }
}

class VelaViewController: UIViewController {
    private var velaRuntime: UnsafeMutableRawPointer?

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .white

        // Initialize Vela runtime
        // velaRuntime = vela_ios_create_runtime()

        // Create root widget and render
        // This would integrate with the Vela renderer
        setupVelaUI()
    }

    private func setupVelaUI() {
        // Placeholder for Vela UI setup
        let label = UILabel()
        label.text = "Hello from Vela!"
        label.textAlignment = .center
        label.frame = view.bounds
        view.addSubview(label)
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        // Update Vela layout if needed
    }
}
"#;

        let main_path = sources_dir.join("main.swift");
        std::fs::write(main_path, main_swift)?;
        println!("📄 Generated main.swift");

        // Generar bridging header si es necesario
        let bridging_header = r#"//
//  VelaApp-Bridging-Header.h
//  VelaApp
//
//  This file is automatically generated. Do not edit.
//

#ifndef VelaApp_Bridging_Header_h
#define VelaApp_Bridging_Header_h

// Vela runtime declarations
void *vela_ios_create_runtime(void);
void vela_ios_destroy_runtime(void *runtime);
void vela_ios_render_widget(void *runtime, const char *widget_json);
void vela_ios_handle_touch_event(void *runtime, const char *event_json);

#endif /* VelaApp_Bridging_Header_h */
"#;

        let bridging_path = sources_dir.join("VelaApp-Bridging-Header.h");
        std::fs::write(bridging_path, bridging_header)?;
        println!("📄 Generated VelaApp-Bridging-Header.h");

        Ok(())
    }

    /// Generate basic iOS app structure
    fn generate_ios_app_structure(&self, output_dir: &Path) -> Result<()> {
        let sources_dir = output_dir.join("Sources");
        std::fs::create_dir_all(&sources_dir)?;

        // Generar Info.plist básico
        let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>VelaApp</string>
    <key>CFBundleIdentifier</key>
    <string>com.vela.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>VelaApp</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSRequiresIPhoneOS</key>
    <true/>
    <key>UIRequiredDeviceCapabilities</key>
    <array>
        <string>armv7</string>
    </array>
    <key>UISupportedInterfaceOrientations</key>
    <array>
        <string>UIInterfaceOrientationPortrait</string>
    </array>
</dict>
</plist>
"#;

        let plist_path = output_dir.join("Info.plist");
        std::fs::write(plist_path, info_plist)?;
        println!("📄 Generated Info.plist");

        Ok(())
    }

    /// Copy compiled bytecode to iOS output
    pub fn copy_compiled_bytecode(&self, ios_output_dir: &Path) -> Result<()> {
        use std::fs;

        let bytecode_dir = ios_output_dir.join("Bytecode");
        fs::create_dir_all(&bytecode_dir)?;

        // Copy all .velac files from target/vela to ios/Bytecode
        let vela_output_dir = self.config.output_dir.join("vela");
        if vela_output_dir.exists() {
            for entry in fs::read_dir(vela_output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("velac") {
                    let file_name = path.file_name().unwrap();
                    let dest = bytecode_dir.join(file_name);
                    fs::copy(&path, &dest)?;
                    println!("📋 Copied bytecode: {}", file_name.to_string_lossy());
                }
            }
        }

        Ok(())
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

    #[test]
    fn test_generate_ios_artifacts_creates_directory_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");

        let config = BuildConfig::new(PathBuf::from("/tmp/project")).with_output_dir(&output_dir);
        let executor = BuildExecutor::new(config);

        // Ejecutar generación de artifacts iOS
        let result = executor.generate_ios_artifacts();
        assert!(result.is_ok(), "generate_ios_artifacts should succeed");

        // Verificar que se creó el directorio ios
        assert!(ios_dir.exists(), "iOS output directory should be created");

        // Verificar archivos generados
        assert!(ios_dir.join("Package.swift").exists(), "Package.swift should be generated");
        assert!(ios_dir.join("Sources").exists(), "Sources directory should be created");
        assert!(ios_dir.join("Sources").join("main.swift").exists(), "main.swift should be generated");
        assert!(ios_dir.join("Sources").join("VelaApp-Bridging-Header.h").exists(), "Bridging header should be generated");
        assert!(ios_dir.join("Info.plist").exists(), "Info.plist should be generated");
    }

    #[test]
    fn test_generate_package_swift_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ios_dir = temp_dir.path().join("ios");

        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        let result = executor.generate_package_swift(&ios_dir);
        assert!(result.is_ok(), "generate_package_swift should succeed");

        let package_path = ios_dir.join("Package.swift");
        assert!(package_path.exists(), "Package.swift should exist");

        let content = std::fs::read_to_string(package_path).unwrap();
        assert!(content.contains("name: \"VelaApp\""), "Package name should be VelaApp");
        assert!(content.contains("platforms: [.iOS(.v14)]"), "Should target iOS 14+");
        assert!(content.contains("executableTarget"), "Should be executable target");
    }

    #[test]
    fn test_generate_ios_wrapper_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ios_dir = temp_dir.path().join("ios");

        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        let result = executor.generate_ios_wrapper(&ios_dir);
        assert!(result.is_ok(), "generate_ios_wrapper should succeed");

        let sources_dir = ios_dir.join("Sources");
        let main_swift = sources_dir.join("main.swift");
        let bridging_header = sources_dir.join("VelaApp-Bridging-Header.h");

        assert!(main_swift.exists(), "main.swift should exist");
        assert!(bridging_header.exists(), "Bridging header should exist");

        let main_content = std::fs::read_to_string(main_swift).unwrap();
        assert!(main_content.contains("@main"), "Should have @main attribute");
        assert!(main_content.contains("UIApplicationMain"), "Should call UIApplicationMain");
        assert!(main_content.contains("VelaViewController"), "Should reference VelaViewController");

        let header_content = std::fs::read_to_string(bridging_header).unwrap();
        assert!(header_content.contains("vela_ios_create_runtime"), "Should declare runtime function");
        assert!(header_content.contains("vela_ios_render_widget"), "Should declare render function");
    }

    #[test]
    fn test_generate_ios_app_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ios_dir = temp_dir.path().join("ios");

        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        let result = executor.generate_ios_app_structure(&ios_dir);
        assert!(result.is_ok(), "generate_ios_app_structure should succeed");

        let plist_path = ios_dir.join("Info.plist");
        assert!(plist_path.exists(), "Info.plist should exist");

        let plist_content = std::fs::read_to_string(plist_path).unwrap();
        assert!(plist_content.contains("VelaApp"), "Should contain app name");
        assert!(plist_content.contains("com.vela.app"), "Should contain bundle identifier");
        assert!(plist_content.contains("UIRequiredDeviceCapabilities"), "Should specify device capabilities");
    }

    #[test]
    fn test_copy_compiled_bytecode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_dir = temp_dir.path().join("output");
        let ios_dir = output_dir.join("ios");
        let vela_dir = output_dir.join("vela");

        // Crear directorio vela y archivo bytecode de prueba
        std::fs::create_dir_all(&vela_dir).unwrap();
        let bytecode_file = vela_dir.join("test.velac");
        std::fs::write(&bytecode_file, "fake bytecode").unwrap();

        let config = BuildConfig::new(output_dir);
        let executor = BuildExecutor::new(config);

        let result = executor.copy_compiled_bytecode(&ios_dir);
        assert!(result.is_ok(), "copy_compiled_bytecode should succeed");

        let copied_file = ios_dir.join("Bytecode").join("test.velac");
        assert!(copied_file.exists(), "Bytecode file should be copied");
        assert_eq!(std::fs::read_to_string(copied_file).unwrap(), "fake bytecode", "Content should match");
    }
}
