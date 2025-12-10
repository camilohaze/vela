/*!
Build configuration
*/

use std::path::PathBuf;

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Project root directory
    pub project_root: PathBuf,

    /// Build in release mode
    pub release: bool,

    /// Target platform
    pub target: Option<String>,

    /// Number of parallel jobs
    pub jobs: Option<usize>,

    /// Output directory
    pub output_dir: PathBuf,

    /// Enable incremental compilation
    pub incremental: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            project_root: std::env::current_dir().unwrap_or(PathBuf::from(".")),
            release: false,
            target: None,
            jobs: Some(num_cpus::get()),
            output_dir: PathBuf::from("target"),
            incremental: true,
        }
    }
}

impl BuildConfig {
    /// Create new build config with project root
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            ..Self::default()
        }
    }

    /// Set release mode
    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    /// Set target platform
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set number of jobs
    pub fn jobs(mut self, jobs: usize) -> Self {
        self.jobs = Some(jobs);
        self
    }

    /// Set number of jobs
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = Some(jobs);
        self
    }

    /// Set output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set incremental compilation
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BuildConfig::default();
        assert!(!config.release);
        assert!(config.incremental);
        assert!(config.jobs.is_some());
    }

    #[test]
    fn test_builder_pattern() {
        let config = BuildConfig::new()
            .with_release(true)
            .with_target("linux")
            .with_jobs(4);

        assert!(config.release);
        assert_eq!(config.target, Some("linux".to_string()));
        assert_eq!(config.jobs, Some(4));
    }

    #[test]
    fn test_with_output_dir() {
        let config = BuildConfig::new().with_output_dir("build");
        assert_eq!(config.output_dir, PathBuf::from("build"));
    }

    #[test]
    fn test_with_incremental() {
        let config = BuildConfig::new().with_incremental(false);
        assert!(!config.incremental);
    }
}

/// num_cpus mock for dependency
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}
