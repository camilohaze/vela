/*!
Build executor with parallel compilation
*/

use crate::build::{BuildCache, BuildConfig, BuildGraph};
use crate::common::Result;
use rayon::prelude::*;

/// Build result
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// Number of modules compiled
    pub modules_compiled: usize,
    /// Number of modules cached
    pub modules_cached: usize,
    /// Build duration in milliseconds
    pub duration_ms: u128,
    /// Success status
    pub success: bool,
}

impl BuildResult {
    /// Create successful build result
    pub fn success(compiled: usize, cached: usize, duration: u128) -> Self {
        Self {
            modules_compiled: compiled,
            modules_cached: cached,
            duration_ms: duration,
            success: true,
        }
    }

    /// Create failed build result
    pub fn failed(duration: u128) -> Self {
        Self {
            modules_compiled: 0,
            modules_cached: 0,
            duration_ms: duration,
            success: false,
        }
    }
}

/// Build executor
pub struct BuildExecutor {
    config: BuildConfig,
    graph: BuildGraph,
    cache: BuildCache,
}

impl BuildExecutor {
    /// Create a new build executor
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            graph: BuildGraph::new(),
            cache: BuildCache::new(),
        }
    }

    /// Execute build
    pub fn execute(&self) -> Result<BuildResult> {
        let start = std::time::Instant::now();

        // Get build levels (topological sort)
        let levels = match self.graph.topological_sort() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Build failed: {}", e);
                return Ok(BuildResult::failed(start.elapsed().as_millis()));
            }
        };

        let mut compiled = 0;
        let mut cached = 0;

        // Compile each level in parallel
        for level in levels {
            let results: Vec<_> = level
                .par_iter()
                .map(|&module_id| {
                    if let Some(module) = self.graph.get_module(module_id) {
                        // Check cache
                        if self.config.incremental && self.cache.is_valid(&module.path).unwrap_or(false) {
                            return (false, true); // Not compiled, but cached
                        }

                        // Compile module (stub)
                        self.compile_module(module_id);
                        (true, false) // Compiled
                    } else {
                        (false, false)
                    }
                })
                .collect();

            for (was_compiled, was_cached) in results {
                if was_compiled {
                    compiled += 1;
                }
                if was_cached {
                    cached += 1;
                }
            }
        }

        let duration = start.elapsed().as_millis();
        Ok(BuildResult::success(compiled, cached, duration))
    }

    /// Compile a single module (stub)
    fn compile_module(&self, _module_id: crate::build::ModuleId) {
        // TODO: Implement actual compilation
        // This would call into vela-compiler crate
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    /// Get mutable reference to graph
    pub fn graph_mut(&mut self) -> &mut BuildGraph {
        &mut self.graph
    }

    /// Get mutable reference to cache
    pub fn cache_mut(&mut self) -> &mut BuildCache {
        &mut self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_result_success() {
        let result = BuildResult::success(10, 5, 100);
        assert!(result.success);
        assert_eq!(result.modules_compiled, 10);
        assert_eq!(result.modules_cached, 5);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_build_result_failed() {
        let result = BuildResult::failed(50);
        assert!(!result.success);
        assert_eq!(result.modules_compiled, 0);
    }

    #[test]
    fn test_new_executor() {
        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        assert_eq!(executor.graph.modules().count(), 0);
    }

    #[test]
    fn test_execute_empty_graph() {
        let config = BuildConfig::default();
        let executor = BuildExecutor::new(config);

        let result = executor.execute().unwrap();
        assert!(result.success);
        assert_eq!(result.modules_compiled, 0);
    }

    #[test]
    fn test_execute_single_module() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        executor.graph_mut().add_module(PathBuf::from("main.vela"));

        let result = executor.execute().unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_execute_with_dependencies() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        let graph = executor.graph_mut();
        let a = graph.add_module(PathBuf::from("a.vela"));
        let b = graph.add_module(PathBuf::from("b.vela"));
        graph.add_dependency(b, a);

        let result = executor.execute().unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_graph_mut() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        executor.graph_mut().add_module(PathBuf::from("test.vela"));
        assert_eq!(executor.graph.modules().count(), 1);
    }

    #[test]
    fn test_cache_mut() {
        let config = BuildConfig::default();
        let mut executor = BuildExecutor::new(config);

        executor.cache_mut().clear();
        assert!(executor.cache.is_empty());
    }
}
