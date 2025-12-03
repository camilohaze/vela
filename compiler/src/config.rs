/*!
# Compiler Configuration

Configuration options for the Vela compiler, controlling optimization levels,
debug output, and performance settings.
*/

use serde::{Deserialize, Serialize};

/// Compiler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Optimization level
    pub optimization: OptimizationLevel,
    /// Debug options
    pub debug: DebugOptions,
    /// Performance settings
    pub performance: PerformanceOptions,
    /// Output format
    pub output_format: OutputFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            optimization: OptimizationLevel::Basic,
            debug: DebugOptions::default(),
            performance: PerformanceOptions::default(),
            output_format: OutputFormat::Binary,
        }
    }
}

/// Optimization levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimizations
    None,
    /// Basic optimizations
    Basic,
    /// Aggressive optimizations
    Aggressive,
    /// Maximum optimizations (may increase compile time)
    Maximum,
}

/// Debug options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugOptions {
    /// Enable debug output for AST
    pub debug_ast: bool,
    /// Enable debug output for lexer
    pub debug_lexer: bool,
    /// Enable debug output for parser
    pub debug_parser: bool,
    /// Enable debug output for semantic analysis
    pub debug_semantic: bool,
    /// Enable debug output for code generation
    pub debug_codegen: bool,
    /// Enable source map generation
    pub source_maps: bool,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            debug_ast: false,
            debug_lexer: false,
            debug_parser: false,
            debug_semantic: false,
            debug_codegen: false,
            source_maps: false,
        }
    }
}

/// Performance options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptions {
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Enable parallel compilation
    pub parallel_compilation: bool,
    /// Maximum memory usage (in MB)
    pub max_memory_mb: usize,
    /// Thread pool size for parallel operations
    pub thread_pool_size: usize,
}

impl Default for PerformanceOptions {
    fn default() -> Self {
        Self {
            enable_simd: true,
            parallel_compilation: true,
            max_memory_mb: 1024, // 1GB
            thread_pool_size: num_cpus::get(),
        }
    }
}

/// Output format for compiled code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Binary bytecode
    Binary,
    /// Text representation
    Text,
    /// LLVM IR
    LlvmIr,
    /// WebAssembly
    Wasm,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.optimization, OptimizationLevel::Basic);
        assert_eq!(config.output_format, OutputFormat::Binary);
    }

    #[test]
    fn test_debug_options_default() {
        let debug = DebugOptions::default();
        assert!(!debug.debug_ast);
        assert!(!debug.debug_lexer);
    }

    #[test]
    fn test_performance_options_default() {
        let perf = PerformanceOptions::default();
        assert!(perf.enable_simd);
        assert!(perf.parallel_compilation);
        assert_eq!(perf.max_memory_mb, 1024);
    }
}