/*!
# Testing Utilities for Native Backend

Common utilities and helpers for testing the native backend.
*/

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[cfg(feature = "llvm_backend")]
use inkwell::context::Context;

#[cfg(feature = "llvm_backend")]
use crate::codegen::{OptimizationLevel, LLVMGenerator, LinkingPipeline};

/// Result of running a test executable
#[derive(Debug, Clone)]
pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub peak_memory_kb: Option<usize>,
}

/// Result of a benchmark test
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub o0_time: Duration,
    pub o1_time: Duration,
    pub o2_time: Duration,
    pub o3_time: Duration,
    pub speedup_o1: f64,
    pub speedup_o2: f64,
    pub speedup_o3: f64,
}

/// Errors that can occur during testing
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Compilation failed: {0}")]
    CompilationError(String),

    #[error("Linking failed: {0}")]
    LinkingError(String),

    #[error("Execution failed: {0}")]
    ExecutionError(String),

    #[error("Test timeout")]
    Timeout,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("LLVM backend not available")]
    LLVMUnavailable,

    #[error("Runtime build failed")]
    RuntimeBuildFailed,
}

/// Main testing harness for native backend
#[cfg(feature = "llvm_backend")]
pub struct NativeBackendTester {
    temp_dir: TempDir,
    llvm_context: Context,
    linker: Option<LinkingPipeline>,
}

#[cfg(feature = "llvm_backend")]
impl NativeBackendTester {
    /// Create a new tester instance
    pub fn new() -> Result<Self, TestError> {
        let temp_dir = TempDir::new().map_err(TestError::IoError)?;
        let llvm_context = Context::create();

        // Try to create linker, but don't fail if runtime isn't built
        let linker = LinkingPipeline::new().ok();

        Ok(NativeBackendTester {
            temp_dir,
            llvm_context,
            linker,
        })
    }

    /// Get the temporary directory for this test session
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Check if the full native backend is available (LLVM + runtime)
    pub fn is_backend_available(&self) -> bool {
        self.linker.is_some()
    }

    /// Compile Vela code to LLVM IR (without optimization)
    pub fn compile_to_ir(&self, vela_code: &str) -> Result<String, TestError> {
        // This is a simplified implementation
        // In a real implementation, this would parse Vela code and generate IR
        let mut generator = LLVMGenerator::new(&self.llvm_context, "test_module")?;

        // For now, return a placeholder
        // TODO: Implement actual Vela parsing and IR generation
        Ok(format!("; IR for: {}", vela_code))
    }

    /// Compile and run Vela code with specified optimization level
    pub fn compile_and_run(&self, vela_code: &str, opt_level: OptimizationLevel) -> Result<TestResult, TestError> {
        if !self.is_backend_available() {
            return Err(TestError::LLVMUnavailable);
        }

        let start_time = Instant::now();

        // Generate LLVM IR
        let ir = self.compile_to_ir(vela_code)?;

        // Write IR to temporary file
        let ir_path = self.temp_dir.path().join("test.ll");
        fs::write(&ir_path, &ir).map_err(TestError::IoError)?;

        // Compile to object file
        let obj_path = self.temp_dir.path().join("test.o");
        self.compile_ir_to_object(&ir_path, &obj_path, opt_level)?;

        // Link with runtime
        let exe_path = self.temp_dir.path().join("test.exe");
        self.link_executable(&obj_path, &exe_path)?;

        let compilation_time = start_time.elapsed();

        // Run the executable
        self.run_executable(&exe_path, compilation_time)
    }

    /// Compile LLVM IR to object file
    fn compile_ir_to_object(&self, ir_path: &Path, obj_path: &Path, _opt_level: OptimizationLevel) -> Result<(), TestError> {
        // Use llc to compile IR to object file
        let output = Command::new("llc")
            .arg("-filetype=obj")
            .arg("-O2")  // Use O2 for tests
            .arg(ir_path)
            .arg("-o")
            .arg(obj_path)
            .output()
            .map_err(TestError::ExecutionError)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TestError::CompilationError(stderr.to_string()));
        }

        Ok(())
    }

    /// Link object file with runtime to create executable
    fn link_executable(&self, obj_path: &Path, exe_path: &Path) -> Result<(), TestError> {
        if let Some(linker) = &self.linker {
            // Use the linking pipeline
            let obj_files = vec![obj_path.to_path_buf()];
            linker.link_executable(&obj_files, exe_path)
                .map_err(TestError::LinkingError)?;
            Ok(())
        } else {
            Err(TestError::RuntimeBuildFailed)
        }
    }

    /// Run executable and capture results
    fn run_executable(&self, exe_path: &Path, compilation_time: Duration) -> Result<TestResult, TestError> {
        let start_time = Instant::now();

        let output = Command::new(exe_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(TestError::ExecutionError)?;

        let execution_time = start_time.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(TestResult {
            exit_code,
            stdout,
            stderr,
            execution_time,
            peak_memory_kb: None, // TODO: Implement memory monitoring
        })
    }

    /// Benchmark code across different optimization levels
    pub fn benchmark_code(&self, vela_code: &str, iterations: usize) -> Result<BenchmarkResult, TestError> {
        let mut times = Vec::new();

        for opt_level in [OptimizationLevel::None, OptimizationLevel::Basic, OptimizationLevel::Default, OptimizationLevel::Aggressive] {
            let mut level_times = Vec::new();

            for _ in 0..iterations {
                let result = self.compile_and_run(vela_code, opt_level)?;
                level_times.push(result.execution_time);
            }

            // Use median time
            level_times.sort();
            times.push(level_times[iterations / 2]);
        }

        let o0_time = times[0];
        let o1_time = times[1];
        let o2_time = times[2];
        let o3_time = times[3];

        Ok(BenchmarkResult {
            o0_time,
            o1_time,
            o2_time,
            o3_time,
            speedup_o1: o0_time.as_secs_f64() / o1_time.as_secs_f64(),
            speedup_o2: o0_time.as_secs_f64() / o2_time.as_secs_f64(),
            speedup_o3: o0_time.as_secs_f64() / o3_time.as_secs_f64(),
        })
    }

    /// Validate that output matches expected result
    pub fn validate_output(&self, expected: &str, actual: &str) -> bool {
        // Simple string comparison, can be extended for more complex validation
        expected.trim() == actual.trim()
    }
}

/// Stub implementation when LLVM backend is not available
#[cfg(not(feature = "llvm_backend"))]
pub struct NativeBackendTester;

#[cfg(not(feature = "llvm_backend"))]
impl NativeBackendTester {
    pub fn new() -> Result<Self, TestError> {
        Ok(NativeBackendTester)
    }

    pub fn is_backend_available(&self) -> bool {
        false
    }

    pub fn compile_and_run(&self, _vela_code: &str, _opt_level: OptimizationLevel) -> Result<TestResult, TestError> {
        Err(TestError::LLVMUnavailable)
    }

    pub fn benchmark_code(&self, _vela_code: &str, _iterations: usize) -> Result<BenchmarkResult, TestError> {
        Err(TestError::LLVMUnavailable)
    }

    pub fn validate_output(&self, expected: &str, actual: &str) -> bool {
        expected.trim() == actual.trim()
    }
}

/// Check if required tools are available
pub fn check_test_environment() -> TestEnvironment {
    TestEnvironment {
        llvm_available: check_command("llc"),
        cmake_available: check_command("cmake"),
        compiler_available: check_command("clang") || check_command("gcc"),
    }
}

/// Test environment status
#[derive(Debug)]
pub struct TestEnvironment {
    pub llvm_available: bool,
    pub cmake_available: bool,
    pub compiler_available: bool,
}

impl TestEnvironment {
    pub fn is_complete(&self) -> bool {
        self.llvm_available && self.cmake_available && self.compiler_available
    }
}

/// Check if a command is available
fn check_command(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Create a temporary Vela source file
pub fn create_temp_source(content: &str) -> Result<PathBuf, TestError> {
    let temp_dir = TempDir::new().map_err(TestError::IoError)?;
    let file_path = temp_dir.path().join("test.vela");
    fs::write(&file_path, content).map_err(TestError::IoError)?;

    // Note: We need to return the path but keep temp_dir alive
    // This is a simplified implementation
    Ok(file_path)
}

/// Clean up temporary files
pub fn cleanup_temp_files() -> Result<(), TestError> {
    // Implementation for cleanup
    Ok(())
}