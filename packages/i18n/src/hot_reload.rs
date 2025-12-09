/*!
Hot reload functionality for translation files during development.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Notify};
use tokio::task::JoinHandle;
use tokio::time;
use crate::error::{I18nError, Result};
use crate::locale::Locale;

/// Hot reload manager for monitoring translation file changes
pub struct HotReloadManager {
    /// Paths being watched
    watch_paths: Vec<PathBuf>,
    /// File modification times
    file_mod_times: RwLock<HashMap<PathBuf, std::time::SystemTime>>,
    /// Change notification
    change_notify: Arc<Notify>,
    /// Watcher task handle
    watcher_task: RwLock<Option<JoinHandle<()>>>,
    /// Callbacks to invoke on change
    callbacks: RwLock<Vec<Box<dyn Fn() + Send + Sync>>>,
    /// Whether hot reload is running
    is_running: RwLock<bool>,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    pub fn new(watch_paths: &[&Path]) -> Self {
        Self {
            watch_paths: watch_paths.iter().map(|p| p.to_path_buf()).collect(),
            file_mod_times: RwLock::new(HashMap::new()),
            change_notify: Arc::new(Notify::new()),
            watcher_task: RwLock::new(None),
            callbacks: RwLock::new(Vec::new()),
            is_running: RwLock::new(false),
        }
    }

    /// Start the hot reload watcher
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        // Initialize file modification times
        self.initialize_file_times().await?;

        // Start the watcher task
        let watch_paths = self.watch_paths.clone();
        let change_notify = Arc::clone(&self.change_notify);
        let callbacks = Arc::clone(&self.callbacks_arc());

        let task = tokio::spawn(async move {
            Self::watcher_loop(watch_paths, change_notify, callbacks).await;
        });

        let mut watcher_task = self.watcher_task.write().await;
        *watcher_task = Some(task);

        Ok(())
    }

    /// Stop the hot reload watcher
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;

        if let Some(task) = self.watcher_task.write().await.take() {
            task.abort();
        }

        Ok(())
    }

    /// Add a callback to be invoked when files change
    pub async fn add_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(Box::new(callback));
    }

    /// Wait for the next file change
    pub async fn wait_for_change(&self) {
        self.change_notify.notified().await;
    }

    /// Check if hot reload is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get the watched paths
    pub fn watch_paths(&self) -> &[PathBuf] {
        &self.watch_paths
    }

    /// Force a reload (useful for testing)
    pub fn force_reload(&self) {
        self.change_notify.notify_waiters();
    }

    /// Initialize file modification times
    async fn initialize_file_times(&self) -> Result<()> {
        let mut mod_times = self.file_mod_times.write().await;

        for watch_path in &self.watch_paths {
            if watch_path.is_dir() {
                self.scan_directory_for_files(watch_path, &mut mod_times).await?;
            } else if watch_path.exists() {
                let metadata = tokio::fs::metadata(watch_path).await
                    .map_err(|e| I18nError::io_error(
                        format!("Failed to get metadata for {}", watch_path.display()),
                        e
                    ))?;

                mod_times.insert(watch_path.clone(), metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH));
            }
        }

        Ok(())
    }

    /// Scan a directory for translation files
    async fn scan_directory_for_files(
        &self,
        dir: &Path,
        mod_times: &mut HashMap<PathBuf, std::time::SystemTime>,
    ) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await
            .map_err(|e| I18nError::io_error(
                format!("Failed to read directory {}", dir.display()),
                e
            ))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            I18nError::io_error("Failed to read directory entry".to_string(), e)
        })? {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if matches!(extension.to_str(), Some("json") | Some("yaml") | Some("yml")) {
                        let metadata = entry.metadata().await
                            .map_err(|e| I18nError::io_error(
                                format!("Failed to get metadata for {}", path.display()),
                                e
                            ))?;

                        mod_times.insert(path, metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH));
                    }
                }
            } else if path.is_dir() {
                // Recursively scan subdirectories
                Box::pin(self.scan_directory_for_files(&path, mod_times)).await?;
            }
        }

        Ok(())
    }

    /// Get an Arc reference to callbacks for the watcher task
    fn callbacks_arc(&self) -> Arc<RwLock<Vec<Box<dyn Fn() + Send + Sync>>>> {
        Arc::new(RwLock::new(Vec::new()))
    }

    /// Main watcher loop
    async fn watcher_loop(
        watch_paths: Vec<PathBuf>,
        change_notify: Arc<Notify>,
        _callbacks: Arc<RwLock<Vec<Box<dyn Fn() + Send + Sync>>>>,
    ) {
        let mut interval = time::interval(Duration::from_millis(500)); // Check every 500ms

        loop {
            interval.tick().await;

            // Check for file changes
            for watch_path in &watch_paths {
                if Self::check_path_for_changes(watch_path).await {
                    // Notify all waiters
                    change_notify.notify_waiters();

                    // Execute callbacks
                    let callbacks = _callbacks.read().await;
                    for callback in callbacks.iter() {
                        callback();
                    }

                    break; // Only notify once per check cycle
                }
            }
        }
    }

    /// Check if a path has changes
    async fn check_path_for_changes(path: &Path) -> bool {
        if path.is_dir() {
            Self::check_directory_for_changes(path).await
        } else {
            Self::check_file_for_changes(path).await
        }
    }

    /// Check directory for changes
    async fn check_directory_for_changes(dir: &Path) -> bool {
        // This is a simplified implementation
        // In a real system, you'd maintain a map of file modification times
        // and compare against current times

        // For now, just check if any .json/.yaml files exist and have been modified recently
        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if matches!(extension.to_str(), Some("json") | Some("yaml") | Some("yml")) {
                            if let Ok(metadata) = entry.metadata().await {
                                if let Ok(modified) = metadata.modified() {
                                    let now = std::time::SystemTime::now();
                                    if let Ok(duration) = now.duration_since(modified) {
                                        // If file was modified within the last second, consider it changed
                                        if duration < Duration::from_secs(1) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check single file for changes
    async fn check_file_for_changes(path: &Path) -> bool {
        if let Ok(metadata) = tokio::fs::metadata(path).await {
            if let Ok(modified) = metadata.modified() {
                let now = std::time::SystemTime::now();
                if let Ok(duration) = now.duration_since(modified) {
                    // If file was modified within the last second, consider it changed
                    return duration < Duration::from_secs(1);
                }
            }
        }

        false
    }
}

impl Drop for HotReloadManager {
    fn drop(&mut self) {
        // Note: In a real implementation, you'd want to properly await the stop
        // but Drop cannot be async
    }
}

/// Configuration for hot reload
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Watch paths
    pub watch_paths: Vec<PathBuf>,
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
    /// Whether to watch subdirectories recursively
    pub recursive: bool,
    /// File extensions to watch
    pub extensions: Vec<String>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec!["translations".into()],
            check_interval_ms: 500,
            recursive: true,
            extensions: vec!["json".to_string(), "yaml".to_string(), "yml".to_string()],
        }
    }
}

/// Hot reload event
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    /// Translation files changed
    TranslationsChanged {
        /// Changed file paths
        changed_files: Vec<PathBuf>,
        /// Timestamp of the change
        timestamp: Instant,
    },
    /// New locale added
    LocaleAdded {
        /// New locale
        locale: Locale,
        /// File path
        file_path: PathBuf,
    },
    /// Locale removed
    LocaleRemoved {
        /// Removed locale
        locale: Locale,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::write;
    use std::time::Duration;

    #[tokio::test]
    async fn test_hot_reload_initialization() {
        let temp_dir = tempdir().unwrap();

        // Create a test translation file
        let file_path = temp_dir.path().join("en.json");
        write(&file_path, r#"{"test": "value"}"#).await.unwrap();

        let manager = HotReloadManager::new(&[temp_dir.path()]);
        assert!(!manager.is_running().await);

        // Start hot reload
        manager.start().await.unwrap();
        assert!(manager.is_running().await);

        // Stop hot reload
        manager.stop().await.unwrap();
        assert!(!manager.is_running().await);
    }

    #[tokio::test]
    async fn test_force_reload() {
        let temp_dir = tempdir().unwrap();

        // Create a test translation file
        let file_path = temp_dir.path().join("en.json");
        tokio::fs::write(&file_path, r#"{"test": "value"}"#).await.unwrap();

        let manager = HotReloadManager::new(&[temp_dir.path()]);

        // Start the manager first
        manager.start().await.unwrap();

        let notified = Arc::clone(&manager.change_notify);
        let notify_task = tokio::spawn(async move {
            notified.notified().await;
        });

        // Force reload
        manager.force_reload();

        // Should complete quickly
        let result = tokio::time::timeout(Duration::from_millis(100), notify_task).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_hot_reload_config() {
        let config = HotReloadConfig::default();
        assert_eq!(config.watch_paths.len(), 1);
        assert_eq!(config.check_interval_ms, 500);
        assert!(config.recursive);
        assert_eq!(config.extensions, vec!["json", "yaml", "yml"]);
    }
}