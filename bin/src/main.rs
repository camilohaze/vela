/*!
Vela CLI - Command Line Interface for the Vela programming language
*/

use clap::{Parser, Subcommand};
use std::process;

/// Vela CLI - Build, run, and manage Vela projects
#[derive(Parser)]
#[command(name = "vela")]
#[command(about = "Vela programming language toolchain")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Vela project
    New {
        /// Project name
        name: String,
        /// Project template
        #[arg(short, long, default_value = "basic")]
        template: String,
        /// Output directory
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Build the current project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// Target platform
        #[arg(long)]
        target: Option<String>,
        /// Number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
    },
    /// Run the project
    Run {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// Arguments to pass to the program
        args: Vec<String>,
    },
    /// Run tests
    Test {
        /// Test filter pattern
        filter: Option<String>,
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Format source code
    Fmt,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { name, template, path } => {
            vela_tooling::cli::commands::execute_new(&name, &template, path.as_deref())
        }
        Commands::Build { release, target, jobs } => {
            vela_tooling::cli::commands::execute_build(release, target.as_deref(), jobs)
        }
        Commands::Run { release, args } => {
            vela_tooling::cli::commands::execute_run(release, &args)
        }
        Commands::Test { filter, release } => {
            vela_tooling::cli::commands::execute_test(filter.as_deref(), release)
        }
        Commands::Fmt => {
            println!("Format command not implemented yet");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}