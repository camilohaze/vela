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
    println!("🧪 Running Vela tests...");
    println!("📋 Configuration:");
    println!("   Release mode: {}", release);
    if let Some(f) = filter {
        println!("   Filter: {}", f);
    }

    // Determinar directorio del proyecto
    let project_root = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?;

    // Buscar archivos de test .vela recursivamente
    let mut test_files = Vec::new();
    fn find_test_files(dir: &std::path::Path, test_files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Only search in directories that contain 'tests' in their path
                if path.components().any(|c| c.as_os_str() == "tests") {
                    find_test_files(&path, test_files)?;
                }
                // Skip all other directories completely
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "vela" {
                    // Check if this is a valid test file: must end with .spec.vela (Angular/NestJS style)
                    let is_test_file = path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.ends_with(".spec.vela"))
                        .unwrap_or(false);

                    if is_test_file {
                        test_files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    find_test_files(&project_root, &mut test_files)?;

    if test_files.is_empty() {
        println!("\n❌ No test files found!");
        println!("   Expected .spec.vela files (Angular/NestJS style)");
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No test files found"
        )));
    }

    println!("\n📂 Found {} test files:", test_files.len());
    for file in &test_files {
        println!("   {}", file.strip_prefix(&project_root).unwrap_or(file).display());
    }

    // Crear directorio de output si no existe
    let output_dir = project_root.join("target");
    std::fs::create_dir_all(&output_dir)?;

    // Ejecutar cada archivo de test
    let mut passed = 0;
    let mut failed = 0;

    for test_file in &test_files {
        println!("\n▶️  Running tests in: {}", test_file.strip_prefix(&project_root).unwrap_or(test_file).display());

        // Compilar el archivo de test usando vela-compiler
        let mut compiler = vela_compiler::Compiler::default();
        let compile_result = compiler.compile_file(test_file);

        let bytecode_bytes = match compile_result {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("❌ Compilation failed: {}", e);
                failed += 1;
                continue;
            }
        };

        // Determinar nombre del archivo bytecode
        let file_stem = test_file.file_stem().unwrap().to_str().unwrap();
        let bytecode_path = output_dir.join(format!("{}.velac", file_stem));

        // Escribir bytecode a archivo
        if let Err(e) = std::fs::write(&bytecode_path, &bytecode_bytes) {
            println!("❌ Failed to write bytecode: {}", e);
            failed += 1;
            continue;
        }

        // Deserializar bytecode
        let bytecode: vela_vm::Bytecode = match bincode::deserialize(&bytecode_bytes) {
            Ok(bc) => bc,
            Err(e) => {
                println!("❌ Failed to deserialize bytecode: {}", e);
                failed += 1;
                continue;
            }
        };

        // Crear VM y ejecutar
        let mut vm = vela_vm::VirtualMachine::new();
        let start_time = std::time::Instant::now();
        let result = vm.execute(&bytecode);
        let duration = start_time.elapsed();

        match result {
            Ok(_) => {
                println!("✅ Tests passed in {} ms", duration.as_millis());
                passed += 1;
            }
            Err(e) => {
                println!("❌ Tests failed after {} ms: {}", duration.as_millis(), e);
                failed += 1;
            }
        }
    }

    // Reporte final
    println!("\n📊 Test Results:");
    println!("   Files: {}", test_files.len());
    println!("   Passed: {} ✅", passed);
    println!("   Failed: {} ❌", failed);

    if failed > 0 {
        println!("\n❌ Some tests failed!");
        Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{} test files failed", failed)
        )))
    } else {
        println!("\n✅ All tests passed!");
        Ok(())
    }
}

/// Execute fmt command
pub fn execute_fmt(check: bool) -> Result<()> {
    println!("🛠️  Running Vela formatter...");
    println!("📋 Configuration:");
    println!("   Check mode: {}", check);

    // Determinar directorio del proyecto
    let project_root = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?;

    // Buscar archivos .vela recursivamente
    let mut vela_files = Vec::new();
    fn find_vela_files(dir: &std::path::Path, vela_files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Skip common directories that shouldn't be formatted
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !["target", ".git", "node_modules", "__pycache__"].contains(&dir_name) {
                    find_vela_files(&path, vela_files)?;
                }
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "vela" {
                    vela_files.push(path);
                }
            }
        }
        Ok(())
    }

    find_vela_files(&project_root, &mut vela_files)?;

    if vela_files.is_empty() {
        println!("\n❌ No .vela files found!");
        return Ok(());
    }

    println!("\n📂 Found {} .vela files:", vela_files.len());
    for file in &vela_files {
        println!("   {}", file.strip_prefix(&project_root).unwrap_or(file).display());
    }

    // Formatear cada archivo
    let mut formatted = 0;
    let mut errors = 0;

    for file_path in &vela_files {
        match format_file(file_path, check) {
            Ok(needs_format) => {
                if needs_format {
                    formatted += 1;
                    if !check {
                        println!("✅ Formatted: {}", file_path.strip_prefix(&project_root).unwrap_or(file_path).display());
                    } else {
                        println!("❌ Needs formatting: {}", file_path.strip_prefix(&project_root).unwrap_or(file_path).display());
                    }
                } else {
                    println!("✅ Already formatted: {}", file_path.strip_prefix(&project_root).unwrap_or(file_path).display());
                }
            }
            Err(e) => {
                println!("❌ Error formatting {}: {}", file_path.strip_prefix(&project_root).unwrap_or(file_path).display(), e);
                errors += 1;
            }
        }
    }

    // Reporte final
    println!("\n📊 Format Results:");
    if check {
        println!("   Files checked: {}", vela_files.len());
        println!("   Need formatting: {} ❌", formatted);
        println!("   Already formatted: {} ✅", vela_files.len() - formatted - errors);
        if errors > 0 {
            println!("   Errors: {} ⚠️", errors);
        }

        if formatted > 0 {
            println!("\n❌ Some files need formatting!");
            println!("   Run 'vela fmt' to format them.");
            Err(crate::common::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{} files need formatting", formatted)
            )))
        } else {
            println!("\n✅ All files are properly formatted!");
            Ok(())
        }
    } else {
        println!("   Files processed: {}", vela_files.len());
        println!("   Formatted: {} ✅", formatted);
        println!("   Already formatted: {} ✅", vela_files.len() - formatted - errors);
        if errors > 0 {
            println!("   Errors: {} ⚠️", errors);
        }

        if errors > 0 {
            println!("\n⚠️  Some files had formatting errors!");
            Err(crate::common::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{} files had formatting errors", errors)
            )))
        } else {
            println!("\n✅ All files formatted successfully!");
            Ok(())
        }
    }
}

/// Format a single Vela file
fn format_file(file_path: &std::path::Path, check: bool) -> Result<bool> {
    // Leer el archivo
    let content = std::fs::read_to_string(file_path)?;

    // Aplicar formato básico
    let formatted = basic_format(&content);

    // Verificar si necesita formato
    let needs_format = content != formatted;

    if !check && needs_format {
        // Escribir el archivo formateado
        std::fs::write(file_path, formatted)?;
    }

    Ok(needs_format)
}

/// Apply basic formatting rules to Vela code
fn basic_format(content: &str) -> String {
    let mut result = String::new();
    let mut indent_level = 0;
    let indent_size = 4;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }

        // Decrease indent for closing braces
        if trimmed.starts_with('}') || trimmed.starts_with(')') || trimmed.starts_with(']') {
            indent_level = indent_level.saturating_sub(1);
        }

        // Add indentation
        for _ in 0..indent_level {
            for _ in 0..indent_size {
                result.push(' ');
            }
        }

        // Add the line content
        result.push_str(trimmed);
        result.push('\n');

        // Increase indent for opening braces
        if trimmed.ends_with('{') || trimmed.ends_with('(') || trimmed.ends_with('[') {
            indent_level += 1;
        }

        // Handle function declarations and other constructs
        if trimmed.starts_with("fn ") ||
           trimmed.starts_with("if ") ||
           trimmed.starts_with("else") ||
           trimmed.starts_with("for ") ||
           trimmed.starts_with("while ") ||
           trimmed.starts_with("match ") {
            if !trimmed.ends_with('{') && !trimmed.ends_with('(') {
                indent_level += 1;
            }
        }

        // Decrease indent after single statements
        if !trimmed.ends_with('{') && !trimmed.ends_with(',') && indent_level > 0 {
            if !(trimmed.starts_with("fn ") ||
                 trimmed.starts_with("if ") ||
                 trimmed.starts_with("else") ||
                 trimmed.starts_with("for ") ||
                 trimmed.starts_with("while ") ||
                 trimmed.starts_with("match ")) {
                indent_level = indent_level.saturating_sub(1);
            }
        }
    }

    // Remove trailing whitespace and empty lines at end
    let mut lines: Vec<&str> = result.lines().collect();
    while let Some(last) = lines.last() {
        if last.trim().is_empty() {
            lines.pop();
        } else {
            break;
        }
    }

    lines.join("\n") + "\n"
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
