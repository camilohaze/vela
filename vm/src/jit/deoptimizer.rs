//! Deoptimizer
//!
//! Handles rollback of failed JIT optimizations, returning to
//! interpreted execution when necessary.

use std::collections::HashSet;
use super::JITResult;

/// Deoptimizer for handling failed JIT optimizations
#[derive(Debug)]
pub struct Deoptimizer {
    /// Functions that have been deoptimized
    deoptimized_functions: HashSet<String>,
    /// Deoptimization statistics
    stats: DeoptimizationStats,
    /// Reasons for deoptimization
    deoptimization_reasons: Vec<DeoptimizationReason>,
}

#[derive(Debug, Clone)]
struct DeoptimizationStats {
    pub total_deoptimizations: usize,
    pub successful_deoptimizations: usize,
    pub failed_deoptimizations: usize,
    pub type_mismatches: usize,
    pub optimization_failures: usize,
    pub memory_pressure: usize,
}

#[derive(Debug, Clone)]
pub enum DeoptimizationReason {
    /// Type mismatch during execution
    TypeMismatch {
        function_id: String,
        expected_type: String,
        actual_type: String,
    },
    /// Optimization assumption violated
    OptimizationFailure {
        function_id: String,
        assumption: String,
    },
    /// Memory pressure requiring cleanup
    MemoryPressure {
        function_id: String,
        memory_used: usize,
    },
    /// Compilation error in optimized code
    CompilationError {
        function_id: String,
        error: String,
    },
    /// Performance regression
    PerformanceRegression {
        function_id: String,
        baseline_time: u64,
        optimized_time: u64,
    },
}

impl Deoptimizer {
    /// Create a new deoptimizer
    pub fn new() -> Self {
        Self {
            deoptimized_functions: HashSet::new(),
            stats: DeoptimizationStats {
                total_deoptimizations: 0,
                successful_deoptimizations: 0,
                failed_deoptimizations: 0,
                type_mismatches: 0,
                optimization_failures: 0,
                memory_pressure: 0,
            },
            deoptimization_reasons: Vec::new(),
        }
    }

    /// Deoptimize a function (remove from JIT cache)
    pub fn deoptimize(&mut self, function_id: &str, reason: DeoptimizationReason) -> JITResult<()> {
        self.stats.total_deoptimizations += 1;

        // Record the reason
        self.deoptimization_reasons.push(reason.clone());

        // Update specific counters
        match &reason {
            DeoptimizationReason::TypeMismatch { .. } => {
                self.stats.type_mismatches += 1;
            }
            DeoptimizationReason::OptimizationFailure { .. } => {
                self.stats.optimization_failures += 1;
            }
            DeoptimizationReason::MemoryPressure { .. } => {
                self.stats.memory_pressure += 1;
            }
            _ => {}
        }

        // Mark function as deoptimized
        self.deoptimized_functions.insert(function_id.to_string());

        // In a real implementation, this would:
        // 1. Remove compiled code from JIT cache
        // 2. Reset profiling counters
        // 3. Notify the VM to use interpreted execution
        // 4. Log the deoptimization event

        self.stats.successful_deoptimizations += 1;
        Ok(())
    }

    /// Check if a function has been deoptimized
    pub fn is_deoptimized(&self, function_id: &str) -> bool {
        self.deoptimized_functions.contains(function_id)
    }

    /// Re-enable a deoptimized function for JIT compilation
    pub fn reenable(&mut self, function_id: &str) -> bool {
        self.deoptimized_functions.remove(function_id)
    }

    /// Get deoptimization reasons for a function
    pub fn get_reasons_for_function(&self, function_id: &str) -> Vec<&DeoptimizationReason> {
        self.deoptimization_reasons
            .iter()
            .filter(|reason| match reason {
                DeoptimizationReason::TypeMismatch { function_id: fid, .. } => fid == function_id,
                DeoptimizationReason::OptimizationFailure { function_id: fid, .. } => fid == function_id,
                DeoptimizationReason::MemoryPressure { function_id: fid, .. } => fid == function_id,
                DeoptimizationReason::CompilationError { function_id: fid, .. } => fid == function_id,
                DeoptimizationReason::PerformanceRegression { function_id: fid, .. } => fid == function_id,
            })
            .collect()
    }

    /// Get deoptimization statistics
    pub fn stats(&self) -> &DeoptimizationStats {
        &self.stats
    }

    /// Clear deoptimization history
    pub fn clear_history(&mut self) {
        self.deoptimized_functions.clear();
        self.deoptimization_reasons.clear();
        self.stats = DeoptimizationStats {
            total_deoptimizations: 0,
            successful_deoptimizations: 0,
            failed_deoptimizations: 0,
            type_mismatches: 0,
            optimization_failures: 0,
            memory_pressure: 0,
        };
    }

    /// Get all deoptimized functions
    pub fn deoptimized_functions(&self) -> &HashSet<String> {
        &self.deoptimized_functions
    }

    /// Check if deoptimization is needed based on performance
    pub fn should_deoptimize(&self, function_id: &str, baseline_time: u64, optimized_time: u64, threshold: f64) -> Option<DeoptimizationReason> {
        if optimized_time as f64 > baseline_time as f64 * threshold {
            Some(DeoptimizationReason::PerformanceRegression {
                function_id: function_id.to_string(),
                baseline_time,
                optimized_time,
            })
        } else {
            None
        }
    }

    /// Handle type mismatch during execution
    pub fn handle_type_mismatch(&mut self, function_id: &str, expected: &str, actual: &str) -> JITResult<()> {
        let reason = DeoptimizationReason::TypeMismatch {
            function_id: function_id.to_string(),
            expected_type: expected.to_string(),
            actual_type: actual.to_string(),
        };
        self.deoptimize(function_id, reason)
    }

    /// Handle optimization assumption violation
    pub fn handle_optimization_failure(&mut self, function_id: &str, assumption: &str) -> JITResult<()> {
        let reason = DeoptimizationReason::OptimizationFailure {
            function_id: function_id.to_string(),
            assumption: assumption.to_string(),
        };
        self.deoptimize(function_id, reason)
    }

    /// Handle memory pressure
    pub fn handle_memory_pressure(&mut self, function_id: &str, memory_used: usize) -> JITResult<()> {
        let reason = DeoptimizationReason::MemoryPressure {
            function_id: function_id.to_string(),
            memory_used,
        };
        self.deoptimize(function_id, reason)
    }
}

impl Default for Deoptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deoptimizer_creation() {
        let deoptimizer = Deoptimizer::new();
        assert!(deoptimizer.deoptimized_functions.is_empty());
        assert!(deoptimizer.deoptimization_reasons.is_empty());
        assert_eq!(deoptimizer.stats().total_deoptimizations, 0);
    }

    #[test]
    fn test_deoptimize_function() {
        let mut deoptimizer = Deoptimizer::new();

        let reason = DeoptimizationReason::TypeMismatch {
            function_id: "test_func".to_string(),
            expected_type: "Number".to_string(),
            actual_type: "String".to_string(),
        };

        let result = deoptimizer.deoptimize("test_func", reason);
        assert!(result.is_ok());
        assert!(deoptimizer.is_deoptimized("test_func"));
        assert_eq!(deoptimizer.stats().total_deoptimizations, 1);
        assert_eq!(deoptimizer.stats().type_mismatches, 1);
    }

    #[test]
    fn test_reenable_function() {
        let mut deoptimizer = Deoptimizer::new();

        let reason = DeoptimizationReason::OptimizationFailure {
            function_id: "test_func".to_string(),
            assumption: "inline cache".to_string(),
        };

        deoptimizer.deoptimize("test_func", reason).unwrap();
        assert!(deoptimizer.is_deoptimized("test_func"));

        let reenabled = deoptimizer.reenable("test_func");
        assert!(reenabled);
        assert!(!deoptimizer.is_deoptimized("test_func"));
    }

    #[test]
    fn test_get_reasons_for_function() {
        let mut deoptimizer = Deoptimizer::new();

        let reason1 = DeoptimizationReason::TypeMismatch {
            function_id: "func1".to_string(),
            expected_type: "Number".to_string(),
            actual_type: "String".to_string(),
        };

        let reason2 = DeoptimizationReason::OptimizationFailure {
            function_id: "func1".to_string(),
            assumption: "monomorphic".to_string(),
        };

        let reason3 = DeoptimizationReason::TypeMismatch {
            function_id: "func2".to_string(),
            expected_type: "Bool".to_string(),
            actual_type: "Null".to_string(),
        };

        deoptimizer.deoptimize("func1", reason1).unwrap();
        deoptimizer.deoptimize("func1", reason2).unwrap();
        deoptimizer.deoptimize("func2", reason3).unwrap();

        let reasons = deoptimizer.get_reasons_for_function("func1");
        assert_eq!(reasons.len(), 2);

        let reasons_func2 = deoptimizer.get_reasons_for_function("func2");
        assert_eq!(reasons_func2.len(), 1);
    }

    #[test]
    fn test_should_deoptimize_performance() {
        let deoptimizer = Deoptimizer::new();

        // Should deoptimize: optimized is 2x slower
        let reason = deoptimizer.should_deoptimize("func1", 100, 250, 1.5);
        assert!(reason.is_some());
        match reason.unwrap() {
            DeoptimizationReason::PerformanceRegression { baseline_time, optimized_time, .. } => {
                assert_eq!(baseline_time, 100);
                assert_eq!(optimized_time, 250);
            }
            _ => panic!("Expected PerformanceRegression"),
        }

        // Should not deoptimize: optimized is faster
        let no_reason = deoptimizer.should_deoptimize("func2", 100, 80, 1.5);
        assert!(no_reason.is_none());
    }

    #[test]
    fn test_handle_type_mismatch() {
        let mut deoptimizer = Deoptimizer::new();

        let result = deoptimizer.handle_type_mismatch("func1", "Number", "String");
        assert!(result.is_ok());
        assert!(deoptimizer.is_deoptimized("func1"));
        assert_eq!(deoptimizer.stats().type_mismatches, 1);
    }

    #[test]
    fn test_handle_optimization_failure() {
        let mut deoptimizer = Deoptimizer::new();

        let result = deoptimizer.handle_optimization_failure("func1", "inline cache");
        assert!(result.is_ok());
        assert!(deoptimizer.is_deoptimized("func1"));
        assert_eq!(deoptimizer.stats().optimization_failures, 1);
    }

    #[test]
    fn test_clear_history() {
        let mut deoptimizer = Deoptimizer::new();

        let reason = DeoptimizationReason::MemoryPressure {
            function_id: "func1".to_string(),
            memory_used: 1000000,
        };

        deoptimizer.deoptimize("func1", reason).unwrap();
        assert_eq!(deoptimizer.stats().total_deoptimizations, 1);

        deoptimizer.clear_history();
        assert_eq!(deoptimizer.stats().total_deoptimizations, 0);
        assert!(deoptimizer.deoptimized_functions.is_empty());
    }
}