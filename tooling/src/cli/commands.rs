/*!
CLI commands implementation
*/

use crate::common::Result;
use crate::build::{BuildConfig, BuildExecutor};
use std::path::PathBuf;

/// Execute new command
pub fn execute_new(name: &str, template: &str, path: Option<&str>) -> Result<()> {
    println!("Creating new project: {} (template: {})", name, template);
    if let Some(p) = path {
        println!("Path: {}", p);
    }
    // TODO: Implement project scaffolding
    Ok(())
}

/// Execute build command
pub fn execute_build(release: bool, target: Option<&str>, jobs: Option<usize>) -> Result<()> {
    println!("🏗️  Building Vela project...");
    println!("📋 Configuration:");
    println!("   Release mode: {}", release);
    if let Some(t) = target {
        println!("   Target: {}", t);
    }
    if let Some(j) = jobs {
        println!("   Parallel jobs: {}", j);
    }

    // Determinar directorio del proyecto (directorio actual)
    let project_root = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?;

    // Crear configuración del build
    let mut config = BuildConfig::new(project_root);
    config = config.release(release);

    if let Some(j) = jobs {
        config = config.jobs(j);
    }

    if let Some(t) = target {
        // TODO: Implementar soporte para diferentes targets
        println!("⚠️  Target specification not yet implemented, using default");
    }

    // Crear y ejecutar el build executor
    let executor = BuildExecutor::new(config);
    let result = executor.execute()?;

    // Reportar resultados
    if result.success {
        println!("\n✅ Build completed successfully!");
        println!("📊 Summary:");
        println!("   Modules compiled: {}", result.modules_compiled);
        println!("   Modules cached: {}", result.modules_cached);
        println!("   Total time: {} ms", result.duration_ms);

        if result.modules_compiled > 0 || result.modules_cached > 0 {
            println!("\n📁 Output directory: target/vela");
        }
    } else {
        println!("\n❌ Build failed after {} ms", result.duration_ms);
        return Err(crate::common::Error::BuildFailed { message: "Build failed".to_string() });
    }

    Ok(())
}

/// Execute run command
pub fn execute_run(release: bool, args: &[String]) -> Result<()> {
    println!("🚀 Running Vela project...");
    println!("📋 Configuration:");
    println!("   Release mode: {}", release);
    if !args.is_empty() {
        println!("   Arguments: {:?}", args);
    }

    // Determinar directorio del proyecto
    let project_root = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?;

    // Determinar directorio de output (target/)
    let output_dir = project_root.join("target");

    // Buscar archivos .velac recursivamente en el directorio target
    let mut main_bytecode_path = None;
    if output_dir.exists() {
        fn find_velac_files(dir: &std::path::Path) -> Result<Option<std::path::PathBuf>> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = find_velac_files(&path)? {
                        return Ok(Some(found));
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("velac") {
                    return Ok(Some(path));
                }
            }
            Ok(None)
        }
        
        main_bytecode_path = find_velac_files(&output_dir)?;
    }

    let main_bytecode_path = match main_bytecode_path {
        Some(path) => path,
        None => {
            println!("\n❌ No compiled bytecode found!");
            println!("   Expected .velac files in: {}", output_dir.display());
            println!("   Make sure to run 'vela build' first.");
            return Err(crate::common::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Compiled bytecode not found. Run 'vela build' first."
            )));
        }
    };

    println!("\n📂 Loading bytecode from: {}", main_bytecode_path.display());

    // Cargar bytecode directamente desde archivo
    let bytecode_bytes = std::fs::read(&main_bytecode_path)
        .map_err(|e| {
            println!("❌ Failed to read bytecode file: {}", e);
            crate::common::Error::Io(e)
        })?;

    // Validar tamaño mínimo
    if bytecode_bytes.len() < 4 {
        println!("❌ Bytecode file too small");
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Bytecode file too small"
        )));
    }

    // Validar magic number
    let magic = u32::from_le_bytes([bytecode_bytes[0], bytecode_bytes[1], bytecode_bytes[2], bytecode_bytes[3]]);
    if magic != vela_vm::Bytecode::MAGIC {
        println!("❌ Invalid magic number: {:08x}, expected {:08x}", magic, vela_vm::Bytecode::MAGIC);
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid magic number: {:08x}, expected {:08x}", magic, vela_vm::Bytecode::MAGIC)
        )));
    }

    // Deserializar bytecode
    let bytecode: vela_vm::Bytecode = bincode::deserialize(&bytecode_bytes)
        .map_err(|e| {
            println!("❌ Failed to deserialize bytecode: {}", e);
            crate::common::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to deserialize bytecode: {}", e)
            ))
        })?;

    println!("✅ Bytecode loaded successfully");
    println!("   Constants: {}", bytecode.constants.len());
    println!("   Code objects: {}", bytecode.code_objects.len());

    // Crear Virtual Machine
    let mut vm = vela_vm::VirtualMachine::new();

    // TODO: Pasar argumentos a la VM cuando esté soportado
    if !args.is_empty() {
        println!("⚠️  Command line arguments not yet supported by VM");
    }

    println!("\n▶️  Executing bytecode...");

    // Ejecutar bytecode
    let start_time = std::time::Instant::now();
    let result = vm.execute(&bytecode);
    let duration = start_time.elapsed();

    match result {
        Ok(value) => {
            println!("\n✅ Execution completed successfully in {} ms", duration.as_millis());
            println!("📤 Result: {:?}", value);
            Ok(())
        }
        Err(e) => {
            println!("\n❌ Execution failed after {} ms", duration.as_millis());
            println!("💥 Error: {}", e);
            Err(crate::common::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("VM execution failed: {}", e)
            )))
        }
    }
}

/// Execute test command
pub fn execute_test(filter: Option<&str>, release: bool) -> Result<()> {
    println!("Running tests (release: {})", release);
    if let Some(f) = filter {
        println!("Filter: {}", f);
    }
    // TODO: Implement test runner
    Ok(())
}

/// Execute fmt command
pub fn execute_fmt(check: bool) -> Result<()> {
    println!("Formatting code (check: {})", check);
    // TODO: Implement formatter
    Ok(())
}

/// Execute lint command
pub fn execute_lint(fix: bool) -> Result<()> {
    println!("Linting code (fix: {})", fix);
    // TODO: Implement linter
    Ok(())
}

/// Execute add command
pub fn execute_add(package: &str, dev: bool) -> Result<()> {
    println!("Adding dependency: {} (dev: {})", package, dev);
    // TODO: Implement dependency addition
    Ok(())
}

/// Execute remove command
pub fn execute_remove(package: &str) -> Result<()> {
    println!("Removing dependency: {}", package);
    // TODO: Implement dependency removal
    Ok(())
}

/// Execute update command
pub fn execute_update(package: Option<&str>) -> Result<()> {
    if let Some(p) = package {
        println!("Updating dependency: {}", p);
    } else {
        println!("Updating all dependencies");
    }
    // TODO: Implement dependency update
    Ok(())
}

/// Execute version command
pub fn execute_version() -> Result<()> {
    println!("vela {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Execute info command
pub fn execute_info() -> Result<()> {
    println!("Project Information");
    println!("-------------------");
    // TODO: Show project details
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_new() {
        let result = execute_new("test", "bin", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_build() {
        let result = execute_build(false, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_run() {
        let result = execute_run(false, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_version() {
        let result = execute_version();
        assert!(result.is_ok());
    }
}
