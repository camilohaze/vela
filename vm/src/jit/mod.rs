//! # JIT Compilation Module
//!
//! Implementation of: VELA-1184 - TASK-175
//! Story: JIT Compilation (Experimental)
//! Date: 2025-12-15
//!
//! Description:
//! Experimental JIT compiler for VelaVM that compiles hotspots
//! to native machine code using LLVM for improved performance.
//!
//! Features:
//! - Hotspot detection through runtime profiling
//! - Dynamic compilation to native code
//! - Code caching and reuse
//! - Deoptimization for failed optimizations

pub mod compiler;
pub mod config;
pub mod deoptimizer;
pub mod profiler;

pub use compiler::JITCompiler;
pub use config::JITConfig;
pub use deoptimizer::Deoptimizer;
pub use profiler::HotspotProfiler;

/// Result type for JIT operations
pub type JITResult<T> = Result<T, JITError>;

/// Errors that can occur during JIT operations
#[derive(Debug, Clone)]
pub enum JITError {
    /// Compilation failed
    CompilationError(String),
    /// Invalid bytecode for compilation
    InvalidBytecode(String),
    /// LLVM backend error
    LLVMError(String),
    /// Cache operation failed
    CacheError(String),
    /// Deoptimization failed
    DeoptimizationError(String),
}

/// Compiled function representation
#[derive(Debug, Clone)]
pub struct CompiledFunction {
    /// Native function pointer
    pub function_ptr: *const (),
    /// Function metadata
    pub metadata: FunctionMetadata,
    /// Compilation timestamp
    pub compiled_at: std::time::Instant,
}

/// Function metadata for JIT compilation
#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    /// Function ID
    pub id: String,
    /// Number of parameters
    pub param_count: usize,
    /// Return type
    pub return_type: String,
    /// Local variable count
    pub local_count: usize,
    /// Estimated execution frequency
    pub call_count: u64,
}

/// JIT compilation statistics
#[derive(Debug, Clone)]
pub struct JITStats {
    /// Total functions compiled
    pub functions_compiled: usize,
    /// Total compilation time
    pub total_compile_time_ms: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Deoptimization events
    pub deoptimizations: usize,
    /// Memory used by compiled code
    pub code_memory_bytes: usize,
}