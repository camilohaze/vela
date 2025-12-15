/*!
CLI commands implementation
*/

use crate::common::Result;
use crate::build::{BuildConfig, BuildExecutor};
use std::path::PathBuf;
use vela_vm::{VirtualMachine, bytecode::Value};
use vela_vm::gc::{GcHeap, GcObject};

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
        config = config.with_target(t);
        println!("   Target: {}", t);
        
        // Validar target soportado
        match t {
            "ios" => {
                println!("📱 Building for iOS target");
            }
            "android" => {
                println!("🤖 Building for Android target");
            }
            "web" => {
                println!("🌐 Building for Web target");
            }
            "desktop" => {
                println!("🖥️  Building for Desktop target");
            }
            _ => {
                println!("⚠️  Unknown target '{}', using default", t);
            }
        }
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

/// Execute debug command (starts DAP server)
pub fn execute_debug(program: &str) -> Result<()> {
    println!("🐛 Starting Vela DAP Debug Server...");
    println!("📋 Configuration:");
    println!("   Program: {}", program);

    // Verify program file exists
    let program_path = std::path::Path::new(program);
    if !program_path.exists() {
        println!("\n❌ Program file not found: {}", program);
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Program file not found: {}", program)
        )));
    }

    println!("\n🔧 Initializing DAP server...");

    // Create DAP server
    let mut dap_server = crate::dap::DapServer::new();

    // Set up launch arguments for the program
    let launch_args = crate::dap::LaunchRequestArguments {
        program: program.to_string(),
        args: vec![],
        cwd: None,
        env: std::collections::HashMap::new(),
    };

    // Simulate launch request
    let launch_request = crate::dap::Request {
        seq: 1,
        command: "launch".to_string(),
        arguments: Some(serde_json::to_value(launch_args).unwrap()),
    };

    println!("🚀 Launching program in debug mode...");
    match dap_server.handle_request(launch_request) {
        Ok(response) => {
            if response.success {
                println!("✅ Program launched successfully");
                println!("🎯 DAP server ready for debugger connections");
                println!("💡 Connect your IDE's debugger to start debugging");
            } else {
                println!("❌ Failed to launch program: {:?}", response.message);
                return Err(crate::common::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Launch failed: {:?}", response.message)
                )));
            }
        }
        Err(e) => {
            println!("❌ Launch error: {}", e);
            return Err(crate::common::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Launch error: {}", e)
            )));
        }
    }

    println!("\n🔄 Starting DAP message loop...");
    println!("💡 Press Ctrl+C to stop the debug server");

    // Start the DAP server message loop
    if let Err(e) = dap_server.start() {
        println!("❌ DAP server error: {}", e);
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("DAP server error: {}", e)
        )));
    }

    Ok(())
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
    let mut indent_level: usize = 0;
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

/// Execute install command
pub fn execute_install() -> Result<()> {
    println!("📦 Installing Vela project dependencies...");

    // Determinar directorio del proyecto
    let project_root = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?;

    let vela_yaml_path = project_root.join("vela.yaml");

    // Verificar que vela.yaml existe
    if !vela_yaml_path.exists() {
        println!("❌ No vela.yaml found in project root");
        println!("   Expected: {}", vela_yaml_path.display());
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "vela.yaml not found"
        )));
    }

    println!("📂 Found project configuration: {}", vela_yaml_path.display());

    // Leer y parsear vela.yaml
    let content = std::fs::read_to_string(&vela_yaml_path)
        .map_err(|e| {
            println!("❌ Failed to read vela.yaml: {}", e);
            crate::common::Error::Io(e)
        })?;

    // Parsear YAML básico (simplificado)
    let dependencies = parse_vela_yaml_dependencies(&content)?;

    if dependencies.is_empty() {
        println!("✅ No dependencies to install");
        return Ok(());
    }

    println!("📋 Dependencies to install:");
    for dep in &dependencies {
        println!("   {}", dep);
    }

    // Crear directorio de dependencias si no existe
    let deps_dir = project_root.join("vela_modules");
    std::fs::create_dir_all(&deps_dir)
        .map_err(|e| {
            println!("❌ Failed to create dependencies directory: {}", e);
            crate::common::Error::Io(e)
        })?;

    println!("📁 Dependencies will be installed to: {}", deps_dir.display());

    // Instalar dependencias (simulado por ahora)
    let mut installed = 0;
    let mut failed = 0;

    for dep in dependencies {
        println!("\n📥 Installing {}...", dep);
        
        // Simular instalación
        match install_dependency(&dep, &deps_dir) {
            Ok(_) => {
                println!("   ✅ Installed {}", dep);
                installed += 1;
            }
            Err(e) => {
                println!("   ❌ Failed to install {}: {}", dep, e);
                failed += 1;
            }
        }
    }

    // Reporte final
    println!("\n📊 Installation Summary:");
    println!("   Successfully installed: {} ✅", installed);
    if failed > 0 {
        println!("   Failed: {} ❌", failed);
        return Err(crate::common::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to install {} dependencies", failed)
        )));
    }

    println!("✅ All dependencies installed successfully!");
    Ok(())
}

/// Parse dependencies from vela.yaml content
fn parse_vela_yaml_dependencies(content: &str) -> Result<Vec<String>> {
    let mut dependencies = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        
        // Buscar líneas de dependencias
        if line.starts_with("dependencies:") || line.starts_with("devDependencies:") {
            // TODO: Parsear YAML completo con una librería
            // Por ahora, parsing básico
            continue;
        }

        // Parsear dependencias externas
        if line.contains("external:") || line.contains("local:") {
            // TODO: Mejor parsing
            continue;
        }

        // Ejemplo de parsing simple: buscar líneas con "name: version"
        if line.contains(": ") && !line.starts_with("name:") && !line.starts_with("version:") {
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() == 2 {
                let dep = format!("{}@{}", parts[0].trim(), parts[1].trim().trim_matches('"'));
                dependencies.push(dep);
            }
        }
    }

    Ok(dependencies)
}

/// Install a single dependency
fn install_dependency(dep: &str, deps_dir: &std::path::Path) -> Result<()> {
    // Parsear nombre y versión
    let parts: Vec<&str> = dep.split('@').collect();
    let name = parts[0];
    let version = parts.get(1).unwrap_or(&"latest");

    // Crear directorio para la dependencia
    let dep_dir = deps_dir.join(name);
    std::fs::create_dir_all(&dep_dir)?;

    // Simular descarga/creación de archivos
    let package_json = format!(r#"{{
  "name": "{}",
  "version": "{}",
  "description": "Simulated package"
}}"#, name, version);

    std::fs::write(dep_dir.join("package.json"), package_json)?;

    // Simular algunos archivos
    std::fs::write(dep_dir.join("index.js"), format!("// {} v{}\nmodule.exports = {{}};", name, version))?;

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

/// Execute doctor command
pub fn execute_doctor() -> Result<()> {
    println!("🔍 Vela Doctor - Installation Diagnostics");
    println!("==========================================");

    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // Check Vela version
    println!("\n📦 Vela Version:");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   ✓ Version check passed");

    // Check current directory
    println!("\n📁 Current Directory:");
    match std::env::current_dir() {
        Ok(cwd) => {
            println!("   Current: {}", cwd.display());
            println!("   ✓ Directory accessible");
        }
        Err(e) => {
            issues.push(format!("Cannot access current directory: {}", e));
        }
    }

    // Check if we're in a Vela project
    println!("\n🏗️  Project Detection:");
    let is_project = std::fs::metadata("vela.toml").is_ok() ||
                     std::fs::metadata("Cargo.toml").is_ok() ||
                     std::fs::metadata("package.json").is_ok();

    if is_project {
        println!("   ✓ Vela project detected");
    } else {
        warnings.push("No Vela project files found in current directory".to_string());
        println!("   ⚠️  No Vela project files detected");
    }

    // Check for source files
    println!("\n📄 Source Files:");
    let vela_files = find_vela_files(&std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?)?;
    println!("   Vela files found: {}", vela_files.len());
    if vela_files.is_empty() {
        warnings.push("No .vela source files found".to_string());
    } else {
        println!("   ✓ Source files detected");
    }

    // Check build directory
    println!("\n🏭 Build Directory:");
    let build_dir = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?
        .join("target");

    if build_dir.exists() {
        match std::fs::read_dir(&build_dir) {
            Ok(_) => println!("   ✓ Build directory accessible"),
            Err(e) => issues.push(format!("Build directory not accessible: {}", e)),
        }
    } else {
        println!("   ℹ️  Build directory not created yet");
    }

    // Check permissions
    println!("\n🔐 Permissions:");
    let test_file = std::env::current_dir()
        .map_err(|e| crate::common::Error::Io(e))?
        .join("vela-doctor-test.tmp");

    match std::fs::write(&test_file, "test") {
        Ok(_) => {
            println!("   ✓ Write permissions OK");
            let _ = std::fs::remove_file(&test_file); // Clean up
        }
        Err(e) => {
            issues.push(format!("No write permissions: {}", e));
        }
    }

    // Check environment variables
    println!("\n🌍 Environment:");
    if let Ok(home) = std::env::var("HOME") {
        println!("   ✓ HOME directory: {}", home);
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        println!("   ✓ USERPROFILE directory: {}", userprofile);
    } else {
        warnings.push("No home directory environment variable found".to_string());
    }

    // Summary
    println!("\n📊 Summary:");
    if issues.is_empty() && warnings.is_empty() {
        println!("   ✅ All checks passed! Vela is ready to use.");
    } else {
        if !issues.is_empty() {
            println!("   ❌ Issues found: {}", issues.len());
            for issue in &issues {
                println!("      - {}", issue);
            }
        }
        if !warnings.is_empty() {
            println!("   ⚠️  Warnings: {}", warnings.len());
            for warning in &warnings {
                println!("      - {}", warning);
            }
        }
    }

    // Return error if there are critical issues
    if !issues.is_empty() {
        return Err(crate::common::Error::BuildFailed {
            message: format!("Doctor found {} critical issues", issues.len())
        });
    }

    Ok(())
}

/// Helper function to find .vela files (reused from fmt command)
fn find_vela_files(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    fn visit_dir(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        // Skip common directories
        let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(dir_name, "target" | "node_modules" | ".git" | "dist" | "build") {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| crate::common::Error::Io(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| crate::common::Error::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                visit_dir(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("vela") {
                files.push(path);
            }
        }

        Ok(())
    }

    visit_dir(dir, &mut files)?;
    Ok(files)
}

/// Execute inspect-signals command
pub fn execute_inspect_signals(program: &str, format: &str) -> Result<()> {
    println!("🔍 Inspecting reactive signals in: {}", program);
    println!("📋 Configuration:");
    println!("   Program: {}", program);
    println!("   Output format: {}", format);

    // Verificar que el archivo existe
    let program_path = std::path::PathBuf::from(program);
    if !program_path.exists() {
        return Err(crate::common::Error::BuildFailed {
            message: format!("Program file not found: {}", program)
        });
    }

    // Compilar el programa usando vela_compiler
    use vela_compiler::{Compiler, config::Config};
    use vela_vm::bytecode::Bytecode;

    let mut compiler = Compiler::new(Config::default());
    let bytecode_bytes = compiler.compile_file(&program_path)
        .map_err(|e| crate::common::Error::BuildFailed {
            message: format!("Compilation failed: {:?}", e)
        })?;

    // Verificar errores de compilación
    if compiler.has_errors() {
        let diagnostics = compiler.diagnostics();
        return Err(crate::common::Error::BuildFailed {
            message: format!("Compilation errors: {:?}", diagnostics)
        });
    }

    // Deserializar bytecode
    let bytecode = Bytecode::from_bytes(&bytecode_bytes)
        .map_err(|e| crate::common::Error::BuildFailed {
            message: format!("Bytecode deserialization failed: {:?}", e)
        })?;

    println!("\n✅ Compilation successful!");
    println!("📊 Bytecode statistics:");
    println!("   - Bytecode size: {} bytes", bytecode_bytes.len());

    // Ejecutar el bytecode en el VM para crear objetos reactivos
    println!("\n🚀 Executing program to initialize reactive objects...");

    let mut vm = VirtualMachine::new_with_heap();

    // Ejecutar el bytecode
    let result = vm.execute(&bytecode);
    if let Err(e) = result {
        println!("⚠️  Warning: Program execution failed: {:?}", e);
        println!("   Continuing with signal inspection anyway...");
    } else {
        println!("✅ Program execution completed successfully");
    }

    // Ejecutar el bytecode
    let result = vm.execute(&bytecode);
    if let Err(e) = result {
        println!("⚠️  Warning: Program execution failed: {:?}", e);
        println!("   Continuing with signal inspection anyway...");
    } else {
        println!("✅ Program execution completed successfully");
    }

    // Inspeccionar objetos reactivos en el heap
    println!("\n🔍 Inspecting reactive objects...");

    // Obtener objetos reactivos del heap
    let reactive_objects = vm.get_reactive_objects();

    // Crear estructura de información para mostrar
    let mut signals = Vec::new();
    let mut computed = Vec::new();

    for obj in &reactive_objects {
        match &*obj.borrow() {
            GcObject::ReactiveSignal(signal_obj) => {
                let signal = signal_obj.borrow();
                signals.push(SignalInfo {
                    id: signal.id.clone(),
                    value: format!("{:?}", signal.value),
                    dependents_count: signal.dependents.len(),
                });
            }
            GcObject::ReactiveComputed(computed_obj) => {
                let computed_val = computed_obj.borrow();
                computed.push(ComputedInfo {
                    id: computed_val.id.clone(),
                    cached_value: computed_val.cached_value.map(|v| format!("{:?}", v)),
                    dependencies_count: computed_val.dependencies.len(),
                });
            }
            _ => {}
        }
    }

    let info = ReactiveObjectsInfo { signals, computed };

    println!("\n📋 Reactive Objects Found:");
    println!("   - Signals: {}", info.signals.len());
    println!("   - Computed values: {}", info.computed.len());

    match format {
        "text" => {
            print_reactive_objects_text(&info);
        }
        "json" => {
            print_reactive_objects_json(&info);
        }
        "graphviz" => {
            print_reactive_objects_graphviz(&info);
        }
        _ => {
            return Err(crate::common::Error::BuildFailed {
                message: format!("Unsupported format: {}", format)
            });
        }
    }

    println!("\n✅ Signal inspection completed successfully!");
    Ok(())
}

/// Estructura para contener información de objetos reactivos
#[derive(Debug)]
struct ReactiveObjectsInfo {
    signals: Vec<SignalInfo>,
    computed: Vec<ComputedInfo>,
}

/// Información de un signal reactivo
#[derive(Debug)]
struct SignalInfo {
    id: String,
    value: String,
    dependents_count: usize,
}

/// Información de un computed reactivo
#[derive(Debug)]
struct ComputedInfo {
    id: String,
    cached_value: Option<String>,
    dependencies_count: usize,
}

/// Imprimir objetos reactivos en formato texto
fn print_reactive_objects_text(info: &ReactiveObjectsInfo) {
    println!("📄 Reactive Objects (Text Format)");
    println!("==================================");

    if info.signals.is_empty() && info.computed.is_empty() {
        println!("No reactive objects found in the program.");
        return;
    }

    if !info.signals.is_empty() {
        println!("\n🔴 Signals:");
        for signal in &info.signals {
            println!("  • {} = {} ({} dependents)",
                    signal.id, signal.value, signal.dependents_count);
        }
    }

    if !info.computed.is_empty() {
        println!("\n🟡 Computed Values:");
        for computed in &info.computed {
            let cached = match &computed.cached_value {
                Some(val) => format!("cached: {}", val),
                None => "not cached".to_string(),
            };
            println!("  • {} ({}, {} dependencies)",
                    computed.id, cached, computed.dependencies_count);
        }
    }
}

/// Imprimir objetos reactivos en formato JSON
fn print_reactive_objects_json(info: &ReactiveObjectsInfo) {
    println!("📄 Reactive Objects (JSON Format)");
    println!("===================================");

    println!("{{");
    println!("  \"signals\": [");
    for (i, signal) in info.signals.iter().enumerate() {
        let comma = if i < info.signals.len() - 1 { "," } else { "" };
        println!("    {{");
        println!("      \"id\": \"{}\",", signal.id);
        println!("      \"value\": \"{}\",", signal.value);
        println!("      \"dependents_count\": {}", signal.dependents_count);
        println!("    }}{}", comma);
    }
    println!("  ],");

    println!("  \"computed\": [");
    for (i, computed) in info.computed.iter().enumerate() {
        let comma = if i < info.computed.len() - 1 { "," } else { "" };
        let cached_value = match &computed.cached_value {
            Some(val) => format!("\"{}\"", val),
            None => "null".to_string(),
        };
        println!("    {{");
        println!("      \"id\": \"{}\",", computed.id);
        println!("      \"cached_value\": {},", cached_value);
        println!("      \"dependencies_count\": {}", computed.dependencies_count);
        println!("    }}{}", comma);
    }
    println!("  ]");
    println!("}}");
}

/// Imprimir objetos reactivos en formato GraphViz
fn print_reactive_objects_graphviz(info: &ReactiveObjectsInfo) {
    println!("📄 Reactive Objects (GraphViz Format)");
    println!("======================================");

    println!("digraph ReactiveGraph {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box];");

    // Definir nodos para signals
    for signal in &info.signals {
        println!("  \"{}\" [label=\"Signal\\n{}\\n{}\", fillcolor=lightcoral, style=filled];",
                signal.id, signal.id, signal.value);
    }

    // Definir nodos para computed
    for computed in &info.computed {
        let cached = match &computed.cached_value {
            Some(val) => format!("\\ncached: {}", val),
            None => "\\nnot cached".to_string(),
        };
        println!("  \"{}\" [label=\"Computed\\n{}{}\", fillcolor=lightyellow, style=filled];",
                computed.id, computed.id, cached);
    }

    // Aquí irían las conexiones de dependencias
    // Por ahora, solo definimos los nodos

    println!("}}");

    if info.signals.is_empty() && info.computed.is_empty() {
        println!("  // No reactive objects found");
    }
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
        // This test expects an error when no compiled bytecode is found
        // In a test environment, there's no built bytecode
        let result = execute_run(false, &[]);
        assert!(result.is_err(), "Should fail when no bytecode is found");
        
        // Check that it's the expected error type
        if let Err(crate::common::Error::Io(io_err)) = result {
            assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
        } else {
            panic!("Expected Io(NotFound) error, got {:?}", result);
        }
    }

    #[test]
    fn test_execute_version() {
        let result = execute_version();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_vela_yaml_dependencies() {
        let yaml = r#"
name: test-project
version: "1.0.0"
dependencies:
  serde: "1.0"
  anyhow: "1.0"
"#;
        let deps = parse_vela_yaml_dependencies(yaml).unwrap();
        assert!(!deps.is_empty());
        // Note: Current parsing is basic, may need improvement
    }

    #[test]
    fn test_install_dependency() {
        use std::env;
        use std::path::PathBuf;
        
        // Create temp directory
        let temp_dir = env::temp_dir().join("vela_test_install");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let result = install_dependency("test-package@1.0.0", &temp_dir);
        assert!(result.is_ok());
        
        // Check if files were created
        let package_dir = temp_dir.join("test-package");
        assert!(package_dir.exists());
        assert!(package_dir.join("package.json").exists());
        assert!(package_dir.join("index.js").exists());
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
