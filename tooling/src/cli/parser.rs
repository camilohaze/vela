/*!
CLI argument parsing using clap
*/

use clap::{Parser, Subcommand};

/// Vela language toolchain
#[derive(Parser, Debug)]
#[command(name = "vela")]
#[command(about = "Vela language toolchain", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new Vela project
    New {
        /// Project name
        name: String,

        /// Project template (bin, lib, web)
        #[arg(short, long, default_value = "bin")]
        template: String,

        /// Target directory
        #[arg(short = 'd', long)]
        path: Option<String>,
    },

    /// Build the project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Target platform
        #[arg(short, long)]
        target: Option<String>,

        /// Number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
    },

    /// Run the project
    Run {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Debug the project using DAP
    Debug {
        /// Program file to debug
        program: String,
    },

    /// Run tests
    Test {
        /// Filter tests by pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Run tests in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Format source code
    Fmt {
        /// Check formatting without applying changes
        #[arg(long)]
        check: bool,
    },

    /// Lint source code
    Lint {
        /// Automatically fix issues
        #[arg(long)]
        fix: bool,
    },

    /// Add a dependency
    Add {
        /// Package name with optional version (e.g., "http@^2.0")
        package: String,

        /// Add as dev dependency
        #[arg(long)]
        dev: bool,
    },

    /// Remove a dependency
    Remove {
        /// Package name
        package: String,
    },

    /// Update dependencies
    Update {
        /// Update specific package (or all if not specified)
        package: Option<String>,
    },

    /// Install dependencies
    Install,

    /// Show version information
    Version,

    /// Show project information
    Info,

    /// Deploy the project to cloud platforms
    Deploy {
        /// Target platform (aws-lambda, vercel, netlify, azure-functions)
        #[arg(short, long, default_value = "aws-lambda")]
        platform: String,

        /// Environment (dev, staging, prod)
        #[arg(short, long, default_value = "dev")]
        env: String,

        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Skip build step
        #[arg(long)]
        no_build: bool,
    },
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_new() {
        let cli = Cli::try_parse_from(&["vela", "new", "my-project"]).unwrap();
        assert!(matches!(cli.command, Commands::New { .. }));

        if let Commands::New { name, .. } = cli.command {
            assert_eq!(name, "my-project");
        }
    }

    #[test]
    fn test_cli_parsing_build() {
        let cli = Cli::try_parse_from(&["vela", "build", "--release"]).unwrap();
        assert!(matches!(cli.command, Commands::Build { .. }));

        if let Commands::Build { release, .. } = cli.command {
            assert!(release);
        }
    }

    #[test]
    fn test_cli_parsing_run() {
        let cli = Cli::try_parse_from(&["vela", "run", "--", "arg1", "arg2"]).unwrap();
        assert!(matches!(cli.command, Commands::Run { .. }));

        if let Commands::Run { args, .. } = cli.command {
            assert_eq!(args, vec!["arg1", "arg2"]);
        }
    }

    #[test]
    fn test_cli_parsing_test() {
        let cli = Cli::try_parse_from(&["vela", "test", "--filter", "unit"]).unwrap();
        assert!(matches!(cli.command, Commands::Test { .. }));
    }

    #[test]
    fn test_cli_parsing_add() {
        let cli = Cli::try_parse_from(&["vela", "add", "http@^2.0"]).unwrap();
        assert!(matches!(cli.command, Commands::Add { .. }));

        if let Commands::Add { package, .. } = cli.command {
            assert_eq!(package, "http@^2.0");
        }
    }

    #[test]
    fn test_cli_verbose_flag() {
        let cli = Cli::try_parse_from(&["vela", "--verbose", "build"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_parsing_deploy() {
        let cli = Cli::try_parse_from(&["vela", "deploy", "--platform", "vercel", "--env", "prod"]).unwrap();
        assert!(matches!(cli.command, Commands::Deploy { .. }));

        if let Commands::Deploy { platform, env, .. } = cli.command {
            assert_eq!(platform, "vercel");
            assert_eq!(env, "prod");
        }
    }
}
