/*!
# Vela CLI

Command-line interface for the Vela programming language.
Provides compilation, execution, and development tools.
*/

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;
use anyhow::{Context, Result};

#[derive(Parser)]
#[command(name = "vela")]
#[command(about = "Vela programming language toolchain")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Vela project from template
    Create {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project template (web, cli, lib, api, module)
        #[arg(short, long, value_enum, default_value = "web")]
        template: ProjectTemplate,
    },

    /// Compile Vela source files
    Build {
        /// Input source files
        #[arg(value_name = "FILES")]
        input: Vec<PathBuf>,

        /// Output file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Optimization level
        #[arg(short = 'O', long, value_enum, default_value = "basic")]
        opt_level: OptLevel,
    },

    /// Run Vela source files (.vela) or bytecode (.velac) files
    Run {
        /// Source file (.vela) or bytecode file (.velac) to execute
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Command-line arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
        
        /// Show VM execution trace (debug)
        #[arg(long)]
        trace: bool,
        
        /// Enable GC statistics
        #[arg(long)]
        gc_stats: bool,
    },

    /// Run Vela tests
    Test {
        /// Test files to run (if not specified, runs all tests in tests/ directory)
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Run tests in verbose mode
        #[arg(short, long)]
        verbose: bool,

        /// Filter tests by pattern
        #[arg(short, long, value_name = "PATTERN")]
        filter: Option<String>,

        /// Show test execution time
        #[arg(long)]
        time: bool,
    },

    /// Check syntax without compiling
    Check {
        /// Source files to check
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    /// Format Vela source code
    Fmt {
        /// Files to format (modifies in place)
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,

        /// Check if files are formatted without modifying
        #[arg(long)]
        check: bool,
    },

    /// Language server for IDE integration
    Lsp,

    /// Development tools
    Dev {
        #[command(subcommand)]
        tool: DevCommands,
    },
}

#[derive(Subcommand)]
enum DevCommands {
    /// Start DevTools UI
    Tools,
    /// Generate documentation
    Doc,
    /// Run benchmarks
    Bench,
}

#[derive(Clone, clap::ValueEnum)]
#[derive(Debug)]
enum OptLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

#[derive(Clone, clap::ValueEnum, Debug)]
enum ProjectTemplate {
    Web,
    Cli,
    Lib,
    Api,
    Module,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name, template } => {
            handle_create(name, template)
        }
        Commands::Build { input, output, opt_level } => {
            handle_build(input, output, opt_level)
        }
        Commands::Run { file, args, trace, gc_stats } => {
            handle_run(file, args, trace, gc_stats)
        }
        Commands::Test { files, verbose, filter, time } => {
            handle_test(files, verbose, filter, time)
        }
        Commands::Check { files } => {
            handle_check(files)
        }
        Commands::Fmt { files, check } => {
            handle_fmt(files, check)
        }
        Commands::Lsp => {
            handle_lsp()
        }
        Commands::Dev { tool } => {
            handle_dev(tool)
        }
    }
}

fn handle_create(name: String, template: ProjectTemplate) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let project_path = Path::new(&name);

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!("Creating new Vela project '{}' with template '{:?}'...", name, template);

    // Create project directory
    fs::create_dir_all(project_path)
        .with_context(|| format!("Failed to create project directory: {}", name))?;

    // Create src directory
    let src_path = project_path.join("src");
    fs::create_dir_all(&src_path)?;

    // Create tests directory
    let tests_path = project_path.join("tests");
    fs::create_dir_all(&tests_path)?;

    // Create docs directory
    let docs_path = project_path.join("docs");
    fs::create_dir_all(&docs_path)?;

    // Generate files based on template
    match template {
        ProjectTemplate::Web => create_web_template(&name, &src_path, &tests_path, &docs_path)?,
        ProjectTemplate::Cli => create_cli_template(&name, &src_path, &tests_path, &docs_path)?,
        ProjectTemplate::Lib => create_lib_template(&name, &src_path, &tests_path, &docs_path)?,
        ProjectTemplate::Api => create_api_template(&name, &src_path, &tests_path, &docs_path)?,
        ProjectTemplate::Module => create_module_template(&name, &src_path, &tests_path, &docs_path)?,
    }

    // Create vela.toml configuration
    create_project_config(&name, &template, project_path)?;

    // Create README.md
    create_readme(&name, &template, project_path)?;

    println!("✅ Project '{}' created successfully!", name);
    println!("📁 Project structure:");
    println!("  {}/", name);
    println!("  ├── src/");
    println!("  ├── tests/");
    println!("  ├── docs/");
    println!("  ├── vela.toml");
    println!("  └── README.md");
    println!();
    println!("🚀 Next steps:");
    println!("  cd {}", name);
    println!("  vela build");
    println!("  vela run");

    Ok(())
}

fn create_web_template(name: &str, src_path: &Path, tests_path: &Path, docs_path: &Path) -> Result<()> {
    // Create main.vela with web app structure
    let main_content = format!(r#"/*
Web Application: {}

A reactive web application built with Vela.
*/

import 'system:ui'
import 'system:reactive'

// Main application component
@component
class {}App extends StatefulWidget {{
    state counter: Number = 0

    fn build() -> Widget {{
        return Column(
            children: [
                Text("Welcome to {}!", style: TextStyle(fontSize: 24)),
                SizedBox(height: 20),
                Text("Counter: ${{this.counter}}", style: TextStyle(fontSize: 18)),
                SizedBox(height: 10),
                Button(
                    text: "Increment",
                    onPressed: () => this.counter = this.counter + 1
                )
            ]
        )
    }}
}}

// Application entry point
fn main() -> void {{
    app = {}App()
    runApp(app)
}}
"#, name, name, name, name);

    fs::write(src_path.join("main.vela"), main_content)?;

    // Create test file
    let test_content = format!(r#"/*
Tests for {} Web Application
*/

import 'system:testing'

@test
fn testAppInitialization() -> void {{
    app = {}App()
    assert(app.counter == 0, "Counter should start at 0")
}}

@test
fn testCounterIncrement() -> void {{
    app = {}App()
    initialCount = app.counter
    app.counter = app.counter + 1
    assert(app.counter == initialCount + 1, "Counter should increment")
}}
"#, name, name, name);

    fs::write(tests_path.join("app_test.vela"), test_content)?;

    // Create documentation
    let doc_content = format!(r#"# {} Web Application

## Overview
This is a reactive web application built with Vela, featuring:
- Reactive UI components
- State management
- Event handling

## Getting Started
1. Build the project: `vela build`
2. Run the application: `vela run`

## Architecture
- `src/main.vela` - Main application component
- `tests/app_test.vela` - Unit tests
"#, name);

    fs::write(docs_path.join("README.md"), doc_content)?;

    Ok(())
}

fn create_cli_template(name: &str, src_path: &Path, tests_path: &Path, docs_path: &Path) -> Result<()> {
    let main_content = format!(r#"/*
Command Line Application: {}

A command-line tool built with Vela.
*/

import 'system:io'

// Main CLI application
fn main() -> void {{
    println("Welcome to {} CLI!")
    println("Arguments: ${{getArgs()}}")

    // Parse command line arguments
    args = getArgs()

    if args.length() == 0 {{
        showHelp()
    }} else {{
        command = args[0]
        match command {{
            "greet" => {{
                name = if args.length() > 1 {{ args[1] }} else {{ "World" }}
                println("Hello, ${{name}}!")
            }}
            "version" => {{
                println("{} v1.0.0")
            }}
            _ => {{
                println("Unknown command: ${{command}}")
                showHelp()
            }}
        }}
    }}
}}

fn showHelp() -> void {{
    println("Usage: {} <command> [options]")
    println()
    println("Commands:")
    println("  greet [name]    Greet someone")
    println("  version         Show version")
    println("  help            Show this help")
}}
"#, name, name, name, name);

    fs::write(src_path.join("main.vela"), main_content)?;

    let test_content = format!(r#"/*
Tests for {} CLI Application
*/

import 'system:testing'

@test
fn testHelpDisplay() -> void {{
    // Test that help can be shown without errors
    // Note: In real implementation, would capture stdout
    assert(true, "Help display test placeholder")
}}

@test
fn testVersionDisplay() -> void {{
    // Test version command
    assert(true, "Version display test placeholder")
}}
"#, name);

    fs::write(tests_path.join("cli_test.vela"), test_content)?;

    let doc_content = format!(r#"# {} CLI Application

## Overview
This is a command-line application built with Vela, featuring:
- Command-line argument parsing
- Multiple commands support
- Help system

## Usage
```bash
# Build and run
vela build
vela run greet "Your Name"
vela run version
vela run help
```

## Architecture
- `src/main.vela` - Main CLI logic
- `tests/cli_test.vela` - Unit tests
"#, name);

    fs::write(docs_path.join("README.md"), doc_content)?;

    Ok(())
}

fn create_lib_template(name: &str, src_path: &Path, tests_path: &Path, docs_path: &Path) -> Result<()> {
    let lib_content = format!(r#"/*
Library: {}

A reusable library built with Vela.
*/

// Public API
public fn add(a: Number, b: Number) -> Number {{
    return a + b
}}

public fn multiply(a: Number, b: Number) -> Number {{
    return a * b
}}

public fn factorial(n: Number) -> Number {{
    if n <= 1 {{
        return 1
    }}
    return n * factorial(n - 1)
}}

// Internal helper (not exported)
fn validatePositive(n: Number) -> Bool {{
    return n > 0
}}
"#, name);

    fs::write(src_path.join("lib.vela"), lib_content)?;

    let test_content = format!(r#"/*
Tests for {} Library
*/

import 'system:testing'
import '../src/lib'

@test
fn testAdd() -> void {{
    result = add(2, 3)
    assert(result == 5, "2 + 3 should equal 5")
}}

@test
fn testMultiply() -> void {{
    result = multiply(4, 5)
    assert(result == 20, "4 * 5 should equal 20")
}}

@test
fn testFactorial() -> void {{
    assert(factorial(0) == 1, "factorial(0) should be 1")
    assert(factorial(1) == 1, "factorial(1) should be 1")
    assert(factorial(5) == 120, "factorial(5) should be 120")
}}
"#, name);

    fs::write(tests_path.join("lib_test.vela"), test_content)?;

    let doc_content = format!(r#"# {} Library

## Overview
This is a reusable library built with Vela, providing mathematical utilities.

## API

### `add(a: Number, b: Number) -> Number`
Returns the sum of two numbers.

### `multiply(a: Number, b: Number) -> Number`
Returns the product of two numbers.

### `factorial(n: Number) -> Number`
Returns the factorial of a number.

## Usage
```vela
import '{}'

result = add(2, 3)  // 5
product = multiply(4, 5)  // 20
fact = factorial(5)  // 120
```

## Architecture
- `src/lib.vela` - Library implementation
- `tests/lib_test.vela` - Unit tests
"#, name, name);

    fs::write(docs_path.join("README.md"), doc_content)?;

    Ok(())
}

fn create_api_template(name: &str, src_path: &Path, tests_path: &Path, docs_path: &Path) -> Result<()> {
    let main_content = format!(r#"/*
API Server: {}

A REST API server built with Vela.
*/

import 'system:http'
import 'system:json'

// API Server
@injectable
service {}ApiService {{
    @get("/api/health")
    async fn health() -> Result<Response> {{
        return Ok(Response.json({{ "status": "healthy", "timestamp": Date.now() }}))
    }}

    @get("/api/users")
    async fn getUsers() -> Result<Response> {{
        users = [
            {{ "id": 1, "name": "Alice" }},
            {{ "id": 2, "name": "Bob" }}
        ]
        return Ok(Response.json(users))
    }}

    @post("/api/users")
    async fn createUser(@body userData: CreateUserDTO) -> Result<Response> {{
        // In real implementation, save to database
        newUser = {{
            "id": Date.now().toNumber(),
            "name": userData.name,
            "email": userData.email
        }}
        return Ok(Response.json(newUser, status: 201))
    }}
}}

// DTOs
struct CreateUserDTO {{
    name: String
    email: String
}}

// Server setup
fn main() -> void {{
    println("Starting {} API Server on port 3000...")

    server = HttpServer()
        .port(3000)
        .service({}ApiService())
        .start()

    println("Server running at http://localhost:3000")
    println("Health check: http://localhost:3000/api/health")
    println("Users API: http://localhost:3000/api/users")
}}
"#, name, name, name, name);

    fs::write(src_path.join("main.vela"), main_content)?;

    let test_content = format!(r#"/*
Tests for {} API Server
*/

import 'system:testing'

@test
fn testHealthEndpoint() -> void {{
    // Test health endpoint structure
    assert(true, "Health endpoint test placeholder")
}}

@test
fn testUserCreation() -> void {{
    // Test user creation logic
    assert(true, "User creation test placeholder")
}}
"#, name);

    fs::write(tests_path.join("api_test.vela"), test_content)?;

    let doc_content = format!(r#"# {} API Server

## Overview
This is a REST API server built with Vela, featuring:
- HTTP endpoints with decorators
- JSON request/response handling
- Dependency injection

## Endpoints

### GET /api/health
Returns server health status.

### GET /api/users
Returns list of users.

### POST /api/users
Creates a new user.

**Request Body:**
```json
{{
  "name": "string",
  "email": "string"
}}
```

## Usage
```bash
# Build and run
vela build
vela run

# Test endpoints
curl http://localhost:3000/api/health
curl http://localhost:3000/api/users
```

## Architecture
- `src/main.vela` - API server implementation
- `tests/api_test.vela` - Unit tests
"#, name);

    fs::write(docs_path.join("README.md"), doc_content)?;

    Ok(())
}

fn create_module_template(name: &str, src_path: &Path, tests_path: &Path, docs_path: &Path) -> Result<()> {
    let module_content = format!(r#"/*
Module: {}

A functional module built with Vela.
*/

@module({{
    declarations: [{}Service],
    exports: [{}Service],
    providers: [{}Service]
}})
module {}Module {{}}

// Service implementation
@injectable
service {}Service {{
    fn greet(name: String) -> String {{
        return "Hello, ${{name}} from {}!"
    }}

    fn calculateSum(numbers: List<Number>) -> Number {{
        return numbers.reduce((acc, n) => acc + n, 0)
    }}

    fn processData(data: Map<String, Any>) -> Map<String, Any> {{
        return data.mapValues(v => {{
            match v {{
                n: Number => n * 2
                s: String => s.toUpperCase()
                _ => v
            }}
        }})
    }}
}}
"#, name, name, name, name, name, name, name);

    fs::write(src_path.join("module.vela"), module_content)?;

    let test_content = format!(r#"/*
Tests for {} Module
*/

import 'system:testing'

@test
fn testGreeting() -> void {{
    service = {}Service()
    result = service.greet("World")
    assert(result.contains("World"), "Greeting should contain the name")
}}

@test
fn testSumCalculation() -> void {{
    service = {}Service()
    numbers = [1, 2, 3, 4, 5]
    result = service.calculateSum(numbers)
    assert(result == 15, "Sum should be 15")
}}
"#, name, name, name);

    fs::write(tests_path.join("module_test.vela"), test_content)?;

    let doc_content = format!(r#"# {} Module

## Overview
This is a functional module built with Vela, demonstrating:
- Module system with dependency injection
- Service architecture
- Functional programming patterns

## API

### `greet(name: String) -> String`
Returns a greeting message.

### `calculateSum(numbers: List<Number>) -> Number`
Calculates the sum of a list of numbers.

### `processData(data: Map<String, Any>) -> Map<String, Any>`
Processes map data by transforming values.

## Usage
```vela
import 'module:{}'

service = inject({}Service)
message = service.greet("Vela")
sum = service.calculateSum([1, 2, 3])
```

## Architecture
- `src/module.vela` - Module and service implementation
- `tests/module_test.vela` - Unit tests
"#, name, name, name);

    fs::write(docs_path.join("README.md"), doc_content)?;

    Ok(())
}

fn create_project_config(name: &str, template: &ProjectTemplate, project_path: &Path) -> Result<()> {
    let template_name = match template {
        ProjectTemplate::Web => "web",
        ProjectTemplate::Cli => "cli",
        ProjectTemplate::Lib => "lib",
        ProjectTemplate::Api => "api",
        ProjectTemplate::Module => "module",
    };

    let config_content = format!(r#"[project]
name = "{}"
version = "0.1.0"
template = "{}"

[dependencies]
# Add your dependencies here
# Example: serde = "1.0"

[build]
# Build configuration
target = "bytecode"
"#, name, template_name);

    fs::write(project_path.join("vela.toml"), config_content)?;
    Ok(())
}

fn create_readme(name: &str, template: &ProjectTemplate, project_path: &Path) -> Result<()> {
    let template_name = match template {
        ProjectTemplate::Web => "web",
        ProjectTemplate::Cli => "cli",
        ProjectTemplate::Lib => "lib",
        ProjectTemplate::Api => "api",
        ProjectTemplate::Module => "module",
    };

    let readme_content = format!(r#"# {}

A Vela project created with template: {}

## Getting Started

1. **Build the project:**
   ```bash
   vela build
   ```

2. **Run the project:**
   ```bash
   vela run
   ```

3. **Run tests:**
   ```bash
   vela test
   ```

## Project Structure

```
{}/
├── src/           # Source code
├── tests/         # Unit tests
├── docs/          # Documentation
├── vela.toml      # Project configuration
└── README.md      # This file
```

## Development

- Format code: `vela fmt`
- Check syntax: `vela check`
- Build optimized: `vela build --opt-level aggressive`

## License

This project is licensed under the MIT License.
"#, name, template_name, name);

    fs::write(project_path.join("README.md"), readme_content)?;
    Ok(())
}

fn handle_build(input: Vec<PathBuf>, output: Option<PathBuf>, opt_level: OptLevel) -> Result<()> {
    use std::fs;
    use std::time::Instant;
    use vela_compiler::{Compiler, config::{Config, OptimizationLevel}};

    if input.is_empty() {
        anyhow::bail!("No input files specified. Usage: vela build <input-files...>");
    }

    println!("Building Vela project...");
    let start = Instant::now();

    // Convert CLI OptLevel to Compiler OptimizationLevel
    let compiler_opt_level = match opt_level {
        OptLevel::None => OptimizationLevel::None,
        OptLevel::Basic => OptimizationLevel::Basic,
        OptLevel::Aggressive => OptimizationLevel::Aggressive,
        OptLevel::Maximum => OptimizationLevel::Maximum,
    };

    // Create compiler config
    let config = Config {
        optimization: compiler_opt_level,
        ..Config::default()
    };

    let mut compiler = Compiler::new(config);
    println!("✅ Compiler created successfully");

    // Determine output file name
    let output_file = match output {
        Some(path) => {
            // Ensure it has .velac extension
            if path.extension().map_or(true, |ext| ext != "velac") {
                path.with_extension("velac")
            } else {
                path
            }
        }
        None => {
            // Use first input file name with .velac extension
            if let Some(first_input) = input.first() {
                first_input.with_extension("velac")
            } else {
                PathBuf::from("output.velac")
            }
        }
    };

    // For now, compile only the first file
    // TODO: Support multiple files and linking
    let first_input = &input[0];

    if !first_input.exists() {
        anyhow::bail!("Input file not found: {}", first_input.display());
    }

    // Compile the file
    println!("Compiling {} -> {}...", first_input.display(), output_file.display());
    println!("🔧 Starting compilation...");
    let bytecode = compiler.compile_file(first_input);
    println!("🔧 Compilation result received");

    let bytecode = match bytecode {
        Ok(bytecode) => {
            println!("✅ Compilation successful");
            bytecode
        }
        Err(e) => {
            println!("❌ Compilation failed with error: {}", e);
            // Show any additional diagnostics
            let diagnostics = compiler.diagnostics();
            if diagnostics.has_errors() {
                eprintln!("\nCompilation errors:");
                for error in diagnostics.errors() {
                    eprintln!("  {} at {}", error.message, error.location);
                }
            }
            anyhow::bail!("Compilation failed");
        }
    };

    // Check for compilation errors (additional check)
    if compiler.has_errors() {
        let diagnostics = compiler.diagnostics();
        eprintln!("❌ Compilation errors:");
        for error in diagnostics.errors() {
            eprintln!("  {} at {}", error.message, error.location);
        }
        for warning in diagnostics.warnings() {
            eprintln!("  ⚠️  {} at {}", warning.message, warning.location);
        }
        anyhow::bail!("Compilation failed with {} errors", diagnostics.errors().count());
    }

    // Write bytecode to output file
    fs::write(&output_file, &bytecode)
        .with_context(|| format!("Failed to write output file: {}", output_file.display()))?;

    let elapsed = start.elapsed();
    println!("✅ Build completed successfully!");
    println!("📦 Output: {} ({:.2} KB)", output_file.display(), bytecode.len() as f64 / 1024.0);
    println!("⏱️  Build time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);

    // Show additional files if multiple inputs (not yet supported)
    if input.len() > 1 {
        println!("⚠️  Note: Multiple input files specified, but only {} was compiled", first_input.display());
        println!("   Multiple file compilation will be supported in a future version.");
    }

    Ok(())
}

fn handle_run(file: PathBuf, args: Vec<String>, trace: bool, gc_stats: bool) -> Result<()> {
    use vela_vm::{Bytecode, VirtualMachine};
    use vela_compiler::{Compiler, config::Config};
    use std::fs;
    use std::time::Instant;
    
    // Verify file exists
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }
    
    let ext = file.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    let bytecode = if ext == "vela" {
        // Compile source file to bytecode
        println!("Compiling {}...", file.display());
        
        let source = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read source file: {}", file.display()))?;
        
        let mut compiler = Compiler::new(Config::default());
        let bytecode_bytes = compiler.compile_string(&source, file.to_string_lossy().as_ref())
            .with_context(|| "Compilation failed")?;
        
        Bytecode::deserialize(&bytecode_bytes)
            .with_context(|| "Failed to deserialize compiled bytecode")?
    } else if ext == "velac" {
        // Load existing bytecode file
        println!("Loading bytecode from {}...", file.display());
        let bytecode_bytes = fs::read(&file)
            .with_context(|| format!("Failed to read bytecode file: {}", file.display()))?;
        
        Bytecode::deserialize(&bytecode_bytes)
            .with_context(|| "Failed to deserialize bytecode (corrupted file?)")?
    } else {
        anyhow::bail!(
            "Unsupported file type: .{}. Expected .vela (source) or .velac (bytecode)",
            ext
        );
    };
    
    if trace {
        println!("\n=== Bytecode Disassembly ===");
        bytecode.disassemble();
        println!("============================\n");
    }
    
    // Create VM and execute
    let mut vm = VirtualMachine::new();
    
    if trace {
        println!("=== VM Execution Trace ===");
    }
    
    let start = Instant::now();
    let result = vm.execute(&bytecode)
        .with_context(|| "VM execution failed")?;
    let elapsed = start.elapsed();
    
    if trace {
        println!("==========================\n");
    }
    
    // Print result
    println!("Result: {}", result);
    println!("Execution time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    
    // Show GC stats if requested
    if gc_stats {
        let stats = vm.gc_stats();
        println!("\n=== GC Statistics ===");
        println!("Objects allocated: {}", stats.objects_allocated);
        println!("Objects freed: {}", stats.objects_freed);
        println!("Collections triggered: {}", stats.collections);
        println!("Bytes allocated: {}", stats.bytes_allocated);
        println!("Bytes freed: {}", stats.bytes_freed);
        println!("=====================");
    }
    
    // Pass args to program (for future use)
    if !args.is_empty() {
        println!("\nNote: Program arguments not yet supported: {:?}", args);
    }
    
    Ok(())
}

fn handle_test(files: Vec<PathBuf>, verbose: bool, filter: Option<String>, time: bool) -> Result<()> {
    use vela_compiler::{Compiler, config::Config};
    use vela_vm::Bytecode;

    println!("Running Vela tests...");

    // Find test files
    let test_files = if files.is_empty() {
        // Find all .vela files in tests/ directory
        let tests_dir = PathBuf::from("tests");
        if !tests_dir.exists() {
            anyhow::bail!("No tests/ directory found. Create tests/ directory with test files.");
        }

        WalkDir::new("tests")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "vela"))
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>()
    } else {
        files
    };

    if test_files.is_empty() {
        println!("No test files found.");
        return Ok(());
    }

    if verbose {
        println!("Found {} test files:", test_files.len());
        for file in &test_files {
            println!("  {}", file.display());
        }
        println!();
    }

    // Create compiler for test execution
    let config = Config::default();
    let mut compiler = Compiler::new(config);

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut total_time = std::time::Duration::new(0, 0);

    // Run each test file
    for test_file in test_files {
        if verbose {
            println!("Running tests in {}...", test_file.display());
        }

        // Check if file matches filter
        if let Some(ref pattern) = filter {
            if !test_file.to_string_lossy().contains(pattern) {
                continue;
            }
        }

        // Compile test file
        let compile_start = Instant::now();
        let bytecode_bytes = match compiler.compile_file(&test_file) {
            Ok(bytecode) => bytecode,
            Err(e) => {
                println!("❌ Failed to compile {}: {}", test_file.display(), e);
                failed_tests += 1;
                continue;
            }
        };
        let compile_time = compile_start.elapsed();

        // Deserialize bytecode
        let bytecode = match Bytecode::deserialize(&bytecode_bytes) {
            Ok(bytecode) => bytecode,
            Err(e) => {
                println!("❌ Failed to deserialize bytecode for {}: {}", test_file.display(), e);
                failed_tests += 1;
                continue;
            }
        };

        // Execute test file
        let exec_start = Instant::now();
        let _result = match vela_vm::VirtualMachine::new().execute(&bytecode) {
            Ok(result) => result,
            Err(e) => {
                println!("❌ Failed to execute {}: {}", test_file.display(), e);
                failed_tests += 1;
                continue;
            }
        };
        let exec_time = exec_start.elapsed();

        // For now, assume tests pass if execution succeeds
        // TODO: Implement proper test framework with assertions
        total_tests += 1;
        passed_tests += 1;

        if verbose || time {
            println!("✅ {} passed ({:.2}ms compile + {:.2}ms exec)",
                    test_file.display(),
                    compile_time.as_secs_f64() * 1000.0,
                    exec_time.as_secs_f64() * 1000.0);
        } else {
            println!("✅ {}", test_file.display());
        }

        total_time += compile_time + exec_time;
    }

    // Print summary
    println!("\n=== Test Results ===");
    println!("Tests run: {}", total_tests);
    println!("Passed: {}", passed_tests);
    println!("Failed: {}", failed_tests);

    if time && total_tests > 0 {
        println!("Total time: {:.2}ms", total_time.as_secs_f64() * 1000.0);
        println!("Average time: {:.2}ms per test",
                (total_time.as_secs_f64() * 1000.0) / total_tests as f64);
    }

    if failed_tests > 0 {
        println!("❌ {} tests failed", failed_tests);
        std::process::exit(1);
    } else {
        println!("✅ All tests passed!");
    }

    Ok(())
}

fn handle_check(_files: Vec<PathBuf>) -> Result<()> {
    println!("Checking Vela files...");
    // TODO: Implement check command
    Ok(())
}

fn handle_fmt(files: Vec<PathBuf>, check: bool) -> Result<()> {
    use std::fs;
    use regex::Regex;

    if files.is_empty() {
        anyhow::bail!("No files specified. Usage: vela fmt <files...>");
    }

    println!("Formatting Vela files...");

    // Basic formatting rules using regex
    let rules = vec![
        // Remove trailing whitespace
        (Regex::new(r"[ \t]+$").unwrap(), ""),
        // Ensure space around return type arrow (more specific - apply first!)
        (Regex::new(r"([^\s])\->([^\s])").unwrap(), "$1 -> $2"),
        // Ensure space after commas (but not in string literals or inside brackets)
        (Regex::new(r",([^\s\n,)])").unwrap(), ", $1"),
        // Ensure space around assignment operators (avoid double spaces)
        (Regex::new(r"([^=!<>\s])=([^=\s])").unwrap(), "$1 = $2"),
        // Ensure space around comparison operators
        (Regex::new(r"([^=!<>\s])==([^=\s])").unwrap(), "$1 == $2"),
        (Regex::new(r"([^=!<>\s])!=([^=\s])").unwrap(), "$1 != $2"),
        (Regex::new(r"([^=!<>\s])<([^=\s])").unwrap(), "$1 < $2"),
        (Regex::new(r"([^=!<>\s])<=([^=\s])").unwrap(), "$1 <= $2"),
        (Regex::new(r"([^=!<>\s])>([^=\s])").unwrap(), "$1 > $2"),
        (Regex::new(r"([^=!<>\s])>=([^=\s])").unwrap(), "$1 >= $2"),
        // Ensure space around arithmetic operators
        (Regex::new(r"([^=!<>\s])\+([^=\s])").unwrap(), "$1 + $2"),
        (Regex::new(r"([^=!<>\s])\-([^=\s])").unwrap(), "$1 - $2"),
        (Regex::new(r"([^=!<>\s])\*([^=\s])").unwrap(), "$1 * $2"),
        (Regex::new(r"([^=!<>\s])\/([^=\s])").unwrap(), "$1 / $2"),
        // Ensure space after colons in type annotations
        (Regex::new(r":([^\s\n:])").unwrap(), ": $1"),
        // Remove excessive blank lines (more than 2 consecutive)
        (Regex::new(r"\n{3,}").unwrap(), "\n\n"),
    ];

    let mut formatted_files = 0;
    let mut error_files = 0;

    for file_path in files {
        if !file_path.exists() {
            eprintln!("❌ File not found: {}", file_path.display());
            error_files += 1;
            continue;
        }

        // Check if it's a .vela file
        let ext = file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if ext != "vela" {
            eprintln!("⚠️  Skipping non-Vela file: {}", file_path.display());
            continue;
        }

        // Read file content
        let content = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Failed to read {}: {}", file_path.display(), e);
                error_files += 1;
                continue;
            }
        };

        // Apply formatting rules
        let mut formatted_content = content.clone();
        for (pattern, replacement) in &rules {
            formatted_content = pattern.replace_all(&formatted_content, *replacement).to_string();
        }

        // Basic indentation fix (simplified - disabled for now)
        // formatted_content = fix_indentation(&formatted_content);

        // Check if file needs formatting
        if formatted_content == content {
            if !check {
                println!("✅ {}", file_path.display());
            }
            continue;
        }

        if check {
            println!("❌ {} needs formatting", file_path.display());
            formatted_files += 1;
        } else {
            // Write formatted content back to file
            match fs::write(&file_path, &formatted_content) {
                Ok(_) => {
                    println!("✅ {}", file_path.display());
                    formatted_files += 1;
                }
                Err(e) => {
                    eprintln!("❌ Failed to write {}: {}", file_path.display(), e);
                    error_files += 1;
                }
            }
        }
    }

    // Summary
    if check {
        if formatted_files > 0 {
            println!("\n❌ {} files need formatting", formatted_files);
            std::process::exit(1);
        } else {
            println!("\n✅ All files are properly formatted");
        }
    } else {
        println!("\n✅ Formatted {} files", formatted_files);
        if error_files > 0 {
            println!("⚠️  {} files had errors", error_files);
            return Err(anyhow::anyhow!("{} files had errors", error_files));
        }
    }

    Ok(())
}

fn fix_indentation(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut current_indent: usize = 0;

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        // Reduce indentation for standalone closing braces
        let mut line_indent = current_indent;
        if trimmed == "}" {
            line_indent = line_indent.saturating_sub(1);
        }

        // Apply indentation
        let indented = format!("{}{}", "    ".repeat(line_indent), trimmed);
        result.push(indented);

        // Increase indent after opening braces
        if trimmed.ends_with('{') {
            current_indent += 1;
        }

        // Decrease indent after standalone closing braces
        if trimmed == "}" {
            current_indent = current_indent.saturating_sub(1);
        }
    }

    result.join("\n")
}

fn handle_lsp() -> Result<()> {
    println!("Starting Vela Language Server...");
    // TODO: Implement LSP
    Ok(())
}

fn handle_dev(_tool: DevCommands) -> Result<()> {
    println!("Starting Vela DevTools...");
    // TODO: Implement dev tools
    Ok(())
}

#[cfg(test)]
mod test_cli_fmt;

#[cfg(test)]
mod test_cli_run;