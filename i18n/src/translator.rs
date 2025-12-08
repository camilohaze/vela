/*!
Main translation API that integrates all i18n components.
*/

use std::collections::HashMap;
use std::path::Path;
use tokio::sync::RwLock;
use crate::error::{I18nError, Result};
use crate::locale::{Locale, LocaleManager};
use crate::loader::{TranslationLoader, TranslationMap};
use crate::interpolator::{Interpolator, InterpolationContext};
use crate::formatter::Formatter;

/// Main translator struct that provides the public API
#[derive(Debug)]
pub struct Translator {
    /// Current locale
    current_locale: RwLock<Locale>,
    /// Fallback locale
    fallback_locale: Locale,
    /// Translation loader
    loader: RwLock<TranslationLoader>,
    /// Interpolator for variables
    interpolator: Interpolator,
    /// Formatter for localized formatting
    formatter: Formatter,
    /// Locale manager
    locale_manager: LocaleManager,
}

impl Translator {
    /// Create a new translator with default settings
    pub fn new() -> Self {
        Self::with_locale(Locale::from("en").unwrap())
    }

    /// Create a new translator with a specific locale
    pub fn with_locale(locale: Locale) -> Self {
        Self {
            current_locale: RwLock::new(locale.clone()),
            fallback_locale: Locale::from("en").unwrap(),
            loader: RwLock::new(TranslationLoader::new("translations")),
            interpolator: Interpolator::new().unwrap(),
            formatter: Formatter::new(),
            locale_manager: LocaleManager::new(),
        }
    }

    /// Set the current locale
    pub async fn set_locale(&self, locale: Locale) {
        let mut current = self.current_locale.write().await;
        *current = locale;
    }

    /// Get the current locale
    pub async fn get_locale(&self) -> Locale {
        self.current_locale.read().await.clone()
    }

    /// Set the fallback locale
    pub fn set_fallback_locale(&mut self, locale: Locale) {
        self.fallback_locale = locale;
    }

    /// Load translations from a directory
    pub async fn load_translations_from_dir<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let mut loader = self.loader.write().await;
        loader.load_all_from_dir(dir).await
    }

    /// Load translations for a specific locale
    pub async fn load_translations_for_locale(&self, locale: &Locale) -> Result<()> {
        let mut loader = self.loader.write().await;
        loader.load_locale(locale).await?;
        Ok(())
    }

    /// Translate a key with optional variables
    pub async fn translate(&self, key: &str, variables: &[(&str, &str)]) -> Result<String> {
        let current_locale = self.current_locale.read().await;
        let translation = self.translate_with_locale(key, &current_locale).await?;

        if variables.is_empty() {
            return Ok(translation);
        }

        // Create interpolation context
        let mut context = InterpolationContext::new(&current_locale);
        for (name, value) in variables {
            context.variables.insert(name.to_string(), value.to_string());
        }

        self.interpolator.interpolate(&translation, &context)
    }

    /// Translate a key for a specific locale
    pub async fn translate_with_locale(&self, key: &str, locale: &Locale) -> Result<String> {
        let loader = self.loader.read().await;

        // Try the requested locale first
        if let Some(translations) = loader.get_cached(locale) {
            if let Some(value) = self.get_nested_value(translations, key) {
                return Ok(value);
            }
        }

        // Try fallback locales
        for fallback_locale in locale.fallback_chain() {
            if let Some(translations) = loader.get_cached(&fallback_locale) {
                if let Some(value) = self.get_nested_value(translations, key) {
                    return Ok(value);
                }
            }
        }

        Err(I18nError::translation_not_found(key.to_string()))
    }

    /// Check if a translation key exists
    pub async fn has_key(&self, key: &str) -> bool {
        let current_locale = self.current_locale.read().await;
        self.has_key_for_locale(key, &current_locale).await
    }

    /// Check if a translation key exists for a specific locale
    pub async fn has_key_for_locale(&self, key: &str, locale: &Locale) -> bool {
        let loader = self.loader.read().await;

        // Try the requested locale first
        if let Some(translations) = loader.get_cached(locale) {
            if self.get_nested_value(translations, key).is_some() {
                return true;
            }
        }

        // Try fallback locales
        for fallback_locale in locale.fallback_chain() {
            if let Some(translations) = loader.get_cached(&fallback_locale) {
                if self.get_nested_value(translations, key).is_some() {
                    return true;
                }
            }
        }

        false
    }

    /// Get all available locales
    pub async fn available_locales(&self) -> Vec<Locale> {
        let loader = self.loader.read().await;
        loader.loaded_locales().into_iter().cloned().collect()
    }

    /// Clear the translation cache
    pub async fn clear_cache(&self) {
        let mut loader = self.loader.write().await;
        loader.clear_cache();
    }

    /// Get a reference to the formatter for direct formatting operations
    pub fn formatter(&self) -> &Formatter {
        &self.formatter
    }

    /// Get a mutable reference to the formatter
    pub fn formatter_mut(&mut self) -> &mut Formatter {
        &mut self.formatter
    }

    /// Get a reference to the locale manager
    pub fn locale_manager(&self) -> &LocaleManager {
        &self.locale_manager
    }

    /// Get a mutable reference to the locale manager
    pub fn locale_manager_mut(&mut self) -> &mut LocaleManager {
        &mut self.locale_manager
    }

    /// Get nested value from translation map using dot notation
    fn get_nested_value(&self, translations: &TranslationMap, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = translations;

        for (i, part) in parts.iter().enumerate() {
            match current.get(*part) {
                Some(serde_json::Value::Object(obj)) => {
                    if i == parts.len() - 1 {
                        // Last part should be a string value
                        return None;
                    }
                    current = obj;
                }
                Some(serde_json::Value::String(s)) => {
                    if i == parts.len() - 1 {
                        return Some(s.clone());
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        None
    }

    /// Translate with pluralization support
    pub async fn translate_plural(&self, key: &str, count: i64, variables: &[(&str, &str)]) -> Result<String> {
        let current_locale = self.current_locale.read().await;

        // Add count to variables
        let mut all_vars = vec![("count", count.to_string().as_str())];
        all_vars.extend_from_slice(variables);

        self.translate(key, &all_vars).await
    }

    /// Get translation with fallback to a default value
    pub async fn translate_or(&self, key: &str, default: &str, variables: &[(&str, &str)]) -> String {
        match self.translate(key, variables).await {
            Ok(translation) => translation,
            Err(_) => {
                if variables.is_empty() {
                    default.to_string()
                } else {
                    // Try to interpolate the default value
                    let current_locale = self.current_locale.read().await;
                    let mut context = InterpolationContext::new(&current_locale);
                    for (name, value) in variables {
                        context.variables.insert(name.to_string(), value.to_string());
                    }

                    self.interpolator.interpolate(default, &context)
                        .unwrap_or_else(|_| default.to_string())
                }
            }
        }
    }
}

/// Builder pattern for Translator configuration
#[derive(Debug)]
pub struct TranslatorBuilder {
    locale: Locale,
    fallback_locale: Option<Locale>,
    translations_dir: String,
}

impl TranslatorBuilder {
    /// Create a new translator builder
    pub fn new() -> Self {
        Self {
            locale: Locale::from("en").unwrap(),
            fallback_locale: None,
            translations_dir: "translations".to_string(),
        }
    }

    /// Set the initial locale
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Set the fallback locale
    pub fn with_fallback_locale(mut self, locale: Locale) -> Self {
        self.fallback_locale = Some(locale);
        self
    }

    /// Set the translations directory
    pub fn with_translations_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.translations_dir = dir.into();
        self
    }

    /// Build the translator
    pub fn build(self) -> Translator {
        let mut translator = Translator::with_locale(self.locale);

        if let Some(fallback) = self.fallback_locale {
            translator.set_fallback_locale(fallback);
        }

        // Update loader directory
        let loader = TranslationLoader::new(&self.translations_dir);
        translator.loader = RwLock::new(loader);

        translator
    }
}

impl Default for TranslatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::write;

    async fn create_test_translator() -> Translator {
        let temp_dir = tempdir().unwrap();

        // Create test translation files
        let en_translations = r#"{
            "greeting": {
                "hello": "Hello",
                "goodbye": "Goodbye"
            },
            "messages": {
                "welcome": "Welcome, {{name}}!",
                "items": "You have {{count, plural, one{# item} other{# items}}}"
            }
        }"#;

        let es_translations = r#"{
            "greeting": {
                "hello": "Hola",
                "goodbye": "Adiós"
            },
            "messages": {
                "welcome": "¡Bienvenido, {{name}}!",
                "items": "Tienes {{count, plural, one{# elemento} other{# elementos}}}"
            }
        }"#;

        write(temp_dir.path().join("en.json"), en_translations).await.unwrap();
        write(temp_dir.path().join("es.json"), es_translations).await.unwrap();

        let translator = TranslatorBuilder::new()
            .with_locale(Locale::from("en").unwrap())
            .with_translations_dir(temp_dir.path().to_string_lossy().to_string())
            .build();

        translator.load_translations_from_dir(temp_dir.path()).await.unwrap();

        translator
    }

    #[tokio::test]
    async fn test_simple_translation() {
        let translator = create_test_translator().await;

        let result = translator.translate("greeting.hello", &[]).await.unwrap();
        assert_eq!(result, "Hello");
    }

    #[tokio::test]
    async fn test_translation_with_variables() {
        let translator = create_test_translator().await;

        let result = translator.translate("messages.welcome", &[("name", "Alice")]).await.unwrap();
        assert_eq!(result, "Welcome, Alice!");
    }

    #[tokio::test]
    async fn test_plural_translation() {
        let translator = create_test_translator().await;

        let result = translator.translate("messages.items", &[("count", "1")]).await.unwrap();
        assert_eq!(result, "You have # item");

        let result = translator.translate("messages.items", &[("count", "5")]).await.unwrap();
        assert_eq!(result, "You have # items");
    }

    #[tokio::test]
    async fn test_locale_switching() {
        let translator = create_test_translator().await;

        // English
        let result = translator.translate("greeting.hello", &[]).await.unwrap();
        assert_eq!(result, "Hello");

        // Switch to Spanish
        translator.set_locale(Locale::from("es").unwrap()).await;
        let result = translator.translate("greeting.hello", &[]).await.unwrap();
        assert_eq!(result, "Hola");
    }

    #[tokio::test]
    async fn test_missing_key() {
        let translator = create_test_translator().await;

        let result = translator.translate("nonexistent.key", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_translate_or_fallback() {
        let translator = create_test_translator().await;

        let result = translator.translate_or("nonexistent.key", "Default message", &[]).await;
        assert_eq!(result, "Default message");
    }

    #[tokio::test]
    async fn test_has_key() {
        let translator = create_test_translator().await;

        assert!(translator.has_key("greeting.hello").await);
        assert!(!translator.has_key("nonexistent.key").await);
    }
}