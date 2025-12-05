/*!
# Vela CLI

Command-line interface for the Vela programming language.
Provides compilation, execution, and development tools.
*/

use clap::{Parser, Subcommand};
use std::path::PathBuf;
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
    /// Compile Vela source files
    Build {
        /// Input source files
        #[arg(short, long, value_name = "FILE")]
        input: Vec<PathBuf>,

        /// Output file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Optimization level
        #[arg(short = 'O', long, value_enum, default_value = "basic")]
        opt_level: OptLevel,
    },

    /// Run Vela bytecode (.velac) files
    Run {
        /// Bytecode file to execute (.velac format)
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
enum OptLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { input, output, opt_level } => {
            handle_build(input, output, opt_level)
        }
        Commands::Run { file, args, trace, gc_stats } => {
            handle_run(file, args, trace, gc_stats)
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

fn handle_build(_input: Vec<PathBuf>, _output: Option<PathBuf>, _opt_level: OptLevel) -> Result<()> {
    println!("Building Vela project...");
    // TODO: Implement build command
    Ok(())
}

fn handle_run(file: PathBuf, args: Vec<String>, trace: bool, gc_stats: bool) -> Result<()> {
    use vela_vm::{Bytecode, VirtualMachine};
    use std::fs;
    use std::time::Instant;
    
    // Verify file exists and has .velac extension
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }
    
    let ext = file.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    if ext != "velac" {
        anyhow::bail!(
            "Expected .velac bytecode file, got .{}\nHint: Use 'vela build' to compile source files first",
            ext
        );
    }
    
    // Load bytecode from file
    println!("Loading bytecode from {}...", file.display());
    let bytecode_bytes = fs::read(&file)
        .with_context(|| format!("Failed to read bytecode file: {}", file.display()))?;
    
    let bytecode = Bytecode::deserialize(&bytecode_bytes)
        .with_context(|| "Failed to deserialize bytecode (corrupted file?)")?;
    
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

fn handle_check(_files: Vec<PathBuf>) -> Result<()> {
    println!("Checking Vela files...");
    // TODO: Implement check command
    Ok(())
}

fn handle_fmt(_files: Vec<PathBuf>, _check: bool) -> Result<()> {
    println!("Formatting Vela files...");
    // TODO: Implement fmt command
    Ok(())
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