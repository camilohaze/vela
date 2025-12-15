//! JIT Configuration
//!
//! Configuration options for the experimental JIT compiler.

/// JIT compiler configuration
#[derive(Debug, Clone)]
pub struct JITConfig {
    /// Whether JIT compilation is enabled
    pub enabled: bool,
    /// Minimum call count to consider a function a hotspot
    pub hotspot_threshold: u32,
    /// Maximum size of the compiled code cache (in bytes)
    pub max_cache_size: usize,
    /// LLVM optimization level (0-3)
    pub optimization_level: u8,
    /// Whether to enable profiling for hotspot detection
    pub enable_profiling: bool,
    /// Whether to enable deoptimization
    pub enable_deoptimization: bool,
    /// Maximum compilation time per function (in ms)
    pub max_compile_time_ms: u64,
    /// Whether to log JIT operations
    pub enable_logging: bool,
}

impl Default for JITConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Experimental, disabled by default
            hotspot_threshold: 1000,
            max_cache_size: 50 * 1024 * 1024, // 50MB
            optimization_level: 2, // Aggressive optimizations
            enable_profiling: true,
            enable_deoptimization: true,
            max_compile_time_ms: 5000, // 5 seconds max per function
            enable_logging: false,
        }
    }
}

impl JITConfig {
    /// Create a performance-oriented configuration
    pub fn performance() -> Self {
        Self {
            enabled: true,
            hotspot_threshold: 500, // Lower threshold for performance
            max_cache_size: 100 * 1024 * 1024, // 100MB
            optimization_level: 3, // Maximum optimizations
            enable_profiling: true,
            enable_deoptimization: true,
            max_compile_time_ms: 10000, // 10 seconds for complex functions
            enable_logging: false,
        }
    }

    /// Create a conservative configuration
    pub fn conservative() -> Self {
        Self {
            enabled: true,
            hotspot_threshold: 5000, // Higher threshold
            max_cache_size: 10 * 1024 * 1024, // 10MB
            optimization_level: 1, // Conservative optimizations
            enable_profiling: true,
            enable_deoptimization: true,
            max_compile_time_ms: 2000, // 2 seconds max
            enable_logging: true, // Enable logging for debugging
        }
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.optimization_level > 3 {
            return Err("Optimization level must be 0-3".to_string());
        }
        if self.hotspot_threshold == 0 {
            return Err("Hotspot threshold must be greater than 0".to_string());
        }
        if self.max_cache_size == 0 {
            return Err("Max cache size must be greater than 0".to_string());
        }
        if self.max_compile_time_ms == 0 {
            return Err("Max compile time must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JITConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.hotspot_threshold, 1000);
        assert_eq!(config.optimization_level, 2);
    }

    #[test]
    fn test_performance_config() {
        let config = JITConfig::performance();
        assert!(config.enabled);
        assert_eq!(config.hotspot_threshold, 500);
        assert_eq!(config.optimization_level, 3);
    }

    #[test]
    fn test_conservative_config() {
        let config = JITConfig::conservative();
        assert!(config.enabled);
        assert_eq!(config.hotspot_threshold, 5000);
        assert_eq!(config.optimization_level, 1);
        assert!(config.enable_logging);
    }

    #[test]
    fn test_config_validation() {
        let mut config = JITConfig::default();
        assert!(config.validate().is_ok());

        config.optimization_level = 5;
        assert!(config.validate().is_err());

        config = JITConfig::default();
        config.hotspot_threshold = 0;
        assert!(config.validate().is_err());
    }
}