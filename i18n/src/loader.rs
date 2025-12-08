/*!
Translation file loading and caching.
*/

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use serde::{Deserialize, Serialize};
use crate::error::{I18nError, Result};
use crate::locale::Locale;

/// Type alias for translation maps
pub type TranslationMap = HashMap<String, serde_json::Value>;

/// Translation loader for loading translation files
#[derive(Debug)]
pub struct TranslationLoader {
    /// Cache of loaded translations
    cache: HashMap<Locale, TranslationMap>,
    /// Base directory for translation files
    base_dir: PathBuf,
}

impl TranslationLoader {
    /// Create a new translation loader
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            cache: HashMap::new(),
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Load translations for a specific locale
    pub async fn load_locale(&mut self, locale: &Locale) -> Result<&TranslationMap> {
        if !self.cache.contains_key(locale) {
            let translations = self.load_translation_file(locale).await?;
            self.cache.insert(locale.clone(), translations);
        }

        Ok(self.cache.get(locale).unwrap())
    }

    /// Load translation file from disk
    async fn load_translation_file(&self, locale: &Locale) -> Result<TranslationMap> {
        // Try different file extensions and formats
        let extensions = ["json", "yaml", "yml"];

        for ext in &extensions {
            let file_path = self.base_dir.join(format!("{}.{}", locale, ext));

            if file_path.exists() {
                return self.load_file(&file_path).await;
            }
        }

        // Try base locale if region-specific file not found
        let base_locale = locale.base_locale();
        if base_locale != *locale {
            for ext in &extensions {
                let file_path = self.base_dir.join(format!("{}.{}", base_locale, ext));

                if file_path.exists() {
                    return self.load_file(&file_path).await;
                }
            }
        }

        Err(I18nError::file_load_error(
            self.base_dir.join(format!("{}.json", locale)),
            std::io::Error::new(std::io::ErrorKind::NotFound, "Translation file not found"),
        ))
    }

    /// Load a specific translation file
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
                    .map_err(|_| I18nError::parse_error(
                        path,
                        serde_json::Error::custom("YAML parsing not implemented in this example")
                    ))
            }
            _ => Err(I18nError::parse_error(
                path,
                serde_json::Error::custom(format!("Unsupported file extension: {}", extension))
            ))
        }
    }

    /// Load all translation files from a directory
    pub async fn load_all_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let dir_path = dir.as_ref();
        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(|e| I18nError::io_error(
                format!("Failed to read directory: {}", dir_path.display()),
                e
            ))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            I18nError::io_error("Failed to read directory entry".to_string(), e)
        })? {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if matches!(extension.to_str(), Some("json") | Some("yaml") | Some("yml")) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(locale) = Locale::from(stem) {
                                // Load the file
                                let translations = self.load_file(&path).await?;
                                self.cache.insert(locale, translations);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get cached translations for a locale
    pub fn get_cached(&self, locale: &Locale) -> Option<&TranslationMap> {
        self.cache.get(locale)
    }

    /// Clear the translation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Check if a locale is loaded
    pub fn is_loaded(&self, locale: &Locale) -> bool {
        self.cache.contains_key(locale)
    }

    /// Get all loaded locales
    pub fn loaded_locales(&self) -> Vec<&Locale> {
        self.cache.keys().collect()
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
    async fn test_load_json_translation() {
        let temp_dir = tempdir().unwrap();
        let loader = TranslationLoader::new(temp_dir.path());

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

        let locale = Locale::from("en").unwrap();
        let translations = loader.load_file(&file_path).await.unwrap();

        assert_eq!(translations["greeting"]["hello"], "Hello");
        assert_eq!(translations["messages"]["welcome"], "Welcome to our app");
    }

    #[tokio::test]
    async fn test_load_all_from_dir() {
        let temp_dir = tempdir().unwrap();
        let mut loader = TranslationLoader::new(temp_dir.path());

        // Create multiple translation files
        let en_content = r#"{"hello": "Hello"}"#;
        let es_content = r#"{"hello": "Hola"}"#;

        write(temp_dir.path().join("en.json"), en_content).await.unwrap();
        write(temp_dir.path().join("es.json"), es_content).await.unwrap();

        loader.load_all_from_dir(temp_dir.path()).await.unwrap();

        let en_locale = Locale::from("en").unwrap();
        let es_locale = Locale::from("es").unwrap();

        assert!(loader.is_loaded(&en_locale));
        assert!(loader.is_loaded(&es_locale));

        let en_translations = loader.get_cached(&en_locale).unwrap();
        assert_eq!(en_translations["hello"], "Hello");
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let loader = TranslationLoader::new(temp_dir.path());

        let locale = Locale::from("nonexistent").unwrap();
        let result = loader.load_translation_file(&locale).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), I18nError::FileLoadError { .. }));
    }
}