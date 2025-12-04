/*!
Project structure detection and management
*/

use crate::common::{Error, FileSystem, Result};
use std::path::{Path, PathBuf};

/// Represents a Vela project
#[derive(Debug, Clone)]
pub struct Project {
    /// Root directory of the project
    pub root: PathBuf,
    /// Path to Vela.toml manifest
    pub manifest_path: PathBuf,
    /// Path to src directory
    pub src_dir: PathBuf,
    /// Path to target directory
    pub target_dir: PathBuf,
}

impl Project {
    /// Find project root from current directory
    pub fn find_root(start: impl AsRef<Path>) -> Result<PathBuf> {
        let mut current = start.as_ref().to_path_buf();

        loop {
            let manifest = current.join("Vela.toml");
            if FileSystem::exists(&manifest) {
                return Ok(current);
            }

            if !current.pop() {
                return Err(Error::project_not_found(start.as_ref()));
            }
        }
    }

    /// Load project from directory
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let root = Self::find_root(&path)?;
        let manifest_path = root.join("Vela.toml");
        let src_dir = root.join("src");
        let target_dir = root.join("target");

        if !FileSystem::exists(&manifest_path) {
            return Err(Error::manifest_not_found(manifest_path));
        }

        Ok(Self {
            root,
            manifest_path,
            src_dir,
            target_dir,
        })
    }

    /// Check if directory is a Vela project
    pub fn is_project(path: impl AsRef<Path>) -> bool {
        let manifest = path.as_ref().join("Vela.toml");
        FileSystem::exists(manifest)
    }

    /// Get all source files
    pub fn source_files(&self) -> Result<Vec<PathBuf>> {
        FileSystem::find_files(&self.src_dir, "vela")
    }

    /// Get cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.target_dir.join("cache")
    }

    /// Get build artifacts directory
    pub fn artifacts_dir(&self) -> PathBuf {
        self.target_dir.join("artifacts")
    }

    /// Ensure project directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        FileSystem::create_dir_all(&self.src_dir)?;
        FileSystem::create_dir_all(&self.target_dir)?;
        FileSystem::create_dir_all(&self.cache_dir())?;
        FileSystem::create_dir_all(&self.artifacts_dir())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        FileSystem::write(temp.path().join("Vela.toml"), "[package]\nname = \"test\"").unwrap();
        FileSystem::create_dir_all(temp.path().join("src")).unwrap();
        temp
    }

    #[test]
    fn test_find_root() {
        let temp = create_test_project();
        let src = temp.path().join("src");

        let root = Project::find_root(&src).unwrap();
        assert_eq!(root, temp.path());
    }

    #[test]
    fn test_find_root_not_found() {
        let temp = TempDir::new().unwrap();
        let result = Project::find_root(temp.path());

        assert!(matches!(result, Err(Error::ProjectNotFound { .. })));
    }

    #[test]
    fn test_from_dir() {
        let temp = create_test_project();
        let project = Project::from_dir(temp.path()).unwrap();

        assert_eq!(project.root, temp.path());
        assert_eq!(project.manifest_path, temp.path().join("Vela.toml"));
        assert_eq!(project.src_dir, temp.path().join("src"));
        assert_eq!(project.target_dir, temp.path().join("target"));
    }

    #[test]
    fn test_is_project() {
        let temp = create_test_project();
        assert!(Project::is_project(temp.path()));

        let not_project = TempDir::new().unwrap();
        assert!(!Project::is_project(not_project.path()));
    }

    #[test]
    fn test_source_files() {
        let temp = create_test_project();
        let src = temp.path().join("src");

        FileSystem::write(src.join("main.vela"), "").unwrap();
        FileSystem::write(src.join("lib.vela"), "").unwrap();

        let project = Project::from_dir(temp.path()).unwrap();
        let files = project.source_files().unwrap();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_cache_dir() {
        let temp = create_test_project();
        let project = Project::from_dir(temp.path()).unwrap();

        assert_eq!(project.cache_dir(), temp.path().join("target/cache"));
    }

    #[test]
    fn test_artifacts_dir() {
        let temp = create_test_project();
        let project = Project::from_dir(temp.path()).unwrap();

        assert_eq!(
            project.artifacts_dir(),
            temp.path().join("target/artifacts")
        );
    }

    #[test]
    fn test_ensure_dirs() {
        let temp = create_test_project();
        let project = Project::from_dir(temp.path()).unwrap();

        project.ensure_dirs().unwrap();

        assert!(FileSystem::is_dir(&project.src_dir));
        assert!(FileSystem::is_dir(&project.target_dir));
        assert!(FileSystem::is_dir(&project.cache_dir()));
        assert!(FileSystem::is_dir(&project.artifacts_dir()));
    }
}
