/*!
Translation file loading and caching with advanced features.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::RwLock as AsyncRwLock;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::error::{I18nError, Result};
use crate::locale::Locale;

/// Type alias for translation maps
pub type TranslationMap = serde_json::Map<String, serde_json::Value>;

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached translations
    translations: TranslationMap,
    /// When this entry was loaded
    loaded_at: SystemTime,
    /// File modification time
    file_modified: Option<SystemTime>,
}

/// LRU Cache for translations
#[derive(Debug)]
struct TranslationCache {
    /// Maximum number of entries
    max_size: usize,
    /// Cache entries with access order
    entries: HashMap<Locale, CacheEntry>,
    /// Access order for LRU eviction
    access_order: Vec<Locale>,
}

impl TranslationCache {
    /// Create a new cache with maximum size
    fn new(max_size: usize) -> Self {
        Self {
            max_size,
            entries: HashMap::new(),
            access_order: Vec::new(),
        }
    }

    /// Get a translation from cache
    fn get(&mut self, locale: &Locale) -> Option<&TranslationMap> {
        if let Some(entry) = self.entries.get(locale) {
            // Update access order
            if let Some(pos) = self.access_order.iter().position(|l| l == locale) {
                self.access_order.remove(pos);
            }
            self.access_order.push(locale.clone());
            Some(&entry.translations)
        } else {
            None
        }
    }

    /// Insert a translation into cache
    fn insert(&mut self, locale: Locale, translations: TranslationMap, file_modified: Option<SystemTime>) {
        let entry = CacheEntry {
            translations,
            loaded_at: SystemTime::now(),
            file_modified,
        };

        // Remove if already exists
        if let Some(pos) = self.access_order.iter().position(|l| l == &locale) {
            self.access_order.remove(pos);
        }

        // Evict if at capacity
        if self.entries.len() >= self.max_size {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.entries.remove(&oldest);
                self.access_order.remove(0);
            }
        }

        self.entries.insert(locale.clone(), entry);
        self.access_order.push(locale);
    }

    /// Check if cache needs refresh based on file modification time
    fn needs_refresh(&self, locale: &Locale, file_modified: SystemTime) -> bool {
        if let Some(entry) = self.entries.get(locale) {
            match entry.file_modified {
                Some(cached_modified) => file_modified > cached_modified,
                None => true, // No modification time cached, refresh
            }
        } else {
            true // Not in cache, needs load
        }
    }

    /// Clear all cache entries
    fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }

    /// Get cache statistics
    fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.entries.len(),
            max_size: self.max_size,
            hit_rate: 0.0, // Would need hit/miss counters for this
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of entries
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

/// Translation loader trait for different loading strategies
#[async_trait]
pub trait TranslationLoader: Send + Sync {
    /// Load translations for a specific locale
    async fn load_translations(&self, locale: &str) -> Result<TranslationMap>;

    /// Load translations with fallback chain
    async fn load_with_fallback(&self, locale: &str, fallbacks: &[String]) -> Result<TranslationMap>;

    /// Get supported file formats
    fn supported_formats(&self) -> Vec<&str>;

    /// Clear any internal caches
    async fn clear_cache(&self);

    /// Check if hot reload is supported
    fn supports_hot_reload(&self) -> bool {
        false
    }
}

/// File system based translation loader with caching and hot reload
#[derive(Debug)]
pub struct FileSystemLoader {
    /// Base directory for translation files
    base_dir: PathBuf,
    /// Translation cache
    cache: Arc<AsyncRwLock<TranslationCache>>,
    /// Whether to enable hot reload
    hot_reload_enabled: bool,
    /// File watching interval for hot reload
    watch_interval: Duration,
}

impl FileSystemLoader {
    /// Create a new file system loader
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cache: Arc::new(AsyncRwLock::new(TranslationCache::new(50))), // Default cache size
            hot_reload_enabled: false,
            watch_interval: Duration::from_secs(2),
        }
    }

    /// Set cache size
    pub async fn with_cache_size(mut self, size: usize) -> Self {
        {
            let mut cache = self.cache.write().await;
            cache.max_size = size;
        }
        self
    }

    /// Enable hot reload
    pub fn with_hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload_enabled = enabled;
        self
    }

    /// Set watch interval for hot reload
    pub fn with_watch_interval(mut self, interval: Duration) -> Self {
        self.watch_interval = interval;
        self
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        cache.stats()
    }

    /// Clear the translation cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Check if a file has been modified since last cache
    async fn needs_refresh(&self, locale: &Locale, file_path: &Path) -> Result<bool> {
        let cache = self.cache.read().await;

        if let Ok(metadata) = fs::metadata(file_path).await {
            if let Ok(modified) = metadata.modified() {
                return Ok(cache.needs_refresh(locale, modified));
            }
        }

        Ok(true) // If we can't get modification time, assume refresh needed
    }

    /// Load translation file with format detection
    async fn load_file(&self, path: &Path) -> Result<TranslationMap> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| I18nError::file_load_error(path, e))?;

        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        match extension {
            "json" => {
                serde_json::from_str(&content)
                    .map_err(|e| I18nError::parse_error(path, e))
            }
            "yaml" | "yml" => {
                serde_yaml::from_str(&content)
                    .map_err(|e| I18nError::ParseError {
                        path: path.to_string_lossy().to_string(),
                        source: serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("YAML parse error: {}", e))),
                    })
            }
            _ => Err(I18nError::ParseError {
                path: path.to_string_lossy().to_string(),
                source: serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unsupported file extension: {}", extension))),
            }),
        }
    }

    /// Find translation file for locale with fallbacks
    fn find_translation_file(&self, locale: &Locale) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let extensions = ["json", "yaml", "yml"];

        // Try full locale (e.g., en-US.json)
        for ext in &extensions {
            candidates.push(self.base_dir.join(format!("{}.{}", locale, ext)));
        }

        // Try base locale (e.g., en.json)
        let base_locale = locale.base_locale();
        if base_locale != *locale {
            for ext in &extensions {
                candidates.push(self.base_dir.join(format!("{}.{}", base_locale, ext)));
            }
        }

        // Try language only (e.g., en.json if not already added)
        if let lang = locale.language() {
            let lang_path = self.base_dir.join(format!("{}.json", lang));
            if !candidates.contains(&lang_path) {
                candidates.push(lang_path);
            }
        }

        candidates
    }

    /// Load translations with caching
    async fn load_with_cache(&self, locale: &Locale) -> Result<TranslationMap> {
        println!("DEBUG: Loading translations for locale {} from base dir: {}", locale, self.base_dir.display());
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(translations) = cache.get(locale) {
                // If hot reload is disabled, return cached version
                if !self.hot_reload_enabled {
                    return Ok(translations.clone());
                }

                // Check if file needs refresh
                let file_paths = self.find_translation_file(locale);
                let needs_refresh = if let Some(file_path) = file_paths.first() {
                    self.needs_refresh(locale, file_path).await.unwrap_or(true)
                } else {
                    true
                };

                if !needs_refresh {
                    return Ok(translations.clone());
                }
            }
        }

        // Load from file
        let file_paths = self.find_translation_file(locale);
        println!("DEBUG: Looking for files: {:?}", file_paths);
        for file_path in file_paths {
            println!("DEBUG: Checking if file exists: {} -> {}", file_path.display(), file_path.exists());
            if file_path.exists() {
                println!("DEBUG: Loading file: {}", file_path.display());
                let translations = self.load_file(&file_path).await?;
                println!("DEBUG: Loaded translations: {:?}", translations);

                // Cache the result
                let file_modified = fs::metadata(&file_path)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok());

                let mut cache = self.cache.write().await;
                cache.insert(locale.clone(), translations.clone(), file_modified);

                return Ok(translations);
            }
        }

        Err(I18nError::file_load_error(
            self.base_dir.join(format!("{}.json", locale)),
            std::io::Error::new(std::io::ErrorKind::NotFound, "Translation file not found"),
        ))
    }
}

#[async_trait]
impl TranslationLoader for FileSystemLoader {
    async fn load_translations(&self, locale: &str) -> Result<TranslationMap> {
        let locale = Locale::from(locale)?;
        self.load_with_cache(&locale).await
    }

    async fn load_with_fallback(&self, locale: &str, fallbacks: &[String]) -> Result<TranslationMap> {
        let mut all_locales = fallbacks.to_vec();
        all_locales.push(locale.to_string());

        let mut combined_translations = TranslationMap::new();

        for locale_str in all_locales {
            match self.load_translations(&locale_str).await {
                Ok(translations) => {
                    // Merge translations (more specific locales override general ones)
                    for (key, value) in translations {
                        combined_translations.insert(key, value);
                    }
                }
                Err(e) => {
                    // Log warning but continue with fallbacks
                    eprintln!("Warning: Failed to load translations for {}: {}", locale_str, e);
                }
            }
        }

        if combined_translations.is_empty() {
            return Err(I18nError::file_load_error(
                PathBuf::from("fallback chain"),
                std::io::Error::new(std::io::ErrorKind::NotFound, "No translations found in fallback chain"),
            ));
        }

        Ok(combined_translations)
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["json", "yaml", "yml"]
    }

    fn supports_hot_reload(&self) -> bool {
        self.hot_reload_enabled
    }

    async fn clear_cache(&self) {
        self.clear_cache().await;
    }
}

/// Translation file structure for JSON/YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    /// The locale this file represents
    pub locale: String,
    /// The actual translations
    pub translations: TranslationMap,
    /// Optional metadata
    #[serde(default)]
    pub metadata: TranslationMetadata,
}

/// Metadata for translation files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranslationMetadata {
    /// Version of the translation file
    pub version: Option<String>,
    /// Author of the translations
    pub author: Option<String>,
    /// Last modified date
    pub last_modified: Option<String>,
    /// Completion percentage
    pub completion: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_filesystem_loader_json() {
        let temp_dir = tempdir().unwrap();
        let loader = FileSystemLoader::new(temp_dir.path());

        // Create a test translation file
        let json_content = r#"{
            "greeting": {
                "hello": "Hello",
                "goodbye": "Goodbye"
            },
            "messages": {
                "welcome": "Welcome to our app"
            }
        }"#;

        let file_path = temp_dir.path().join("en.json");
        write(&file_path, json_content).await.unwrap();

        let translations = loader.load_translations("en").await.unwrap();

        assert_eq!(translations["greeting"]["hello"], "Hello");
        assert_eq!(translations["messages"]["welcome"], "Welcome to our app");
    }

    #[tokio::test]
    async fn test_filesystem_loader_yaml() {
        let temp_dir = tempdir().unwrap();
        let loader = FileSystemLoader::new(temp_dir.path());

        // Create a test YAML translation file
        let yaml_content = r#"
greeting:
  hello: "Hello"
  goodbye: "Goodbye"
messages:
  welcome: "Welcome to our app"
"#;

        let file_path = temp_dir.path().join("en.yaml");
        write(&file_path, yaml_content).await.unwrap();

        let translations = loader.load_translations("en").await.unwrap();

        assert_eq!(translations["greeting"]["hello"], "Hello");
        assert_eq!(translations["messages"]["welcome"], "Welcome to our app");
    }

    #[tokio::test]
    async fn test_fallback_chain() {
        let temp_dir = tempdir().unwrap();
        let loader = FileSystemLoader::new(temp_dir.path());

        // Create base language file
        let en_content = r#"{"hello": "Hello", "common": "Common"}"#;
        fs::write(temp_dir.path().join("en.json"), en_content).await.unwrap();

        // Create specific locale file
        let en_us_content = r#"{"hello": "Hi", "specific": "US specific"}"#;
        fs::write(temp_dir.path().join("en-US.json"), en_us_content).await.unwrap();

        // Test fallback chain: en-US -> en
        let translations = loader.load_with_fallback("en-US", &["en".to_string()]).await.unwrap();

        assert_eq!(translations["hello"], "Hi"); // From en-US
        assert_eq!(translations["specific"], "US specific"); // From en-US
        assert_eq!(translations["common"], "Common"); // From en fallback
    }

    #[tokio::test]
    async fn test_caching() {
        let temp_dir = tempdir().unwrap();
        let mut loader = FileSystemLoader::new(temp_dir.path());
        loader = loader.with_cache_size(10).await;

        // Create a test file
        let content = r#"{"test": "value"}"#;
        let file_path = temp_dir.path().join("en.json");
        fs::write(&file_path, content).await.unwrap();

        // Load once
        let translations1 = loader.load_translations("en").await.unwrap();
        assert_eq!(translations1["test"], "value");

        // Modify file
        let new_content = r#"{"test": "updated"}"#;
        fs::write(&file_path, new_content).await.unwrap();

        // Load again (should return cached version if hot reload disabled)
        let translations2 = loader.load_translations("en").await.unwrap();
        assert_eq!(translations2["test"], "value"); // Still old value

        // Clear cache and load again
        loader.clear_cache().await;
        let translations3 = loader.load_translations("en").await.unwrap();
        assert_eq!(translations3["test"], "updated"); // New value
    }

    #[tokio::test]
    async fn test_hot_reload() {
        let temp_dir = tempdir().unwrap();
        let loader = FileSystemLoader::new(temp_dir.path())
            .with_hot_reload(true)
            .with_watch_interval(Duration::from_millis(100));

        // Create a test file
        let content = r#"{"test": "value"}"#;
        let file_path = temp_dir.path().join("en.json");
        fs::write(&file_path, content).await.unwrap();

        // Load once
        let translations1 = loader.load_translations("en").await.unwrap();
        assert_eq!(translations1["test"], "value");

        // Modify file
        let new_content = r#"{"test": "updated"}"#;
        fs::write(&file_path, new_content).await.unwrap();

        // Clear cache to simulate hot reload detection
        loader.clear_cache().await;

        // Load again (should reload from file)
        let translations2 = loader.load_translations("en").await.unwrap();
        assert_eq!(translations2["test"], "updated");
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let loader = FileSystemLoader::new(temp_dir.path());

        // Try to load a valid locale but with no files present
        let result = loader.load_translations("en").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), I18nError::FileLoadError { .. }));
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        let temp_dir = tempdir().unwrap();
        let loader = FileSystemLoader::new(temp_dir.path());

        // Create a file with unsupported extension
        let content = "test content";
        let file_path = temp_dir.path().join("en.txt");
        fs::write(&file_path, content).await.unwrap();

        let result = loader.load_file(&file_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let temp_dir = tempdir().unwrap();
        let mut loader = FileSystemLoader::new(temp_dir.path());
        loader = loader.with_cache_size(5).await;

        let stats = loader.cache_stats().await;
        assert_eq!(stats.max_size, 5);
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    async fn test_supported_formats() {
        let loader = FileSystemLoader::new("/tmp");
        let formats = loader.supported_formats();
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"yaml"));
        assert!(formats.contains(&"yml"));
    }
}