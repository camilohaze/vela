/*!
Main translation API that integrates all i18n components.
*/

use std::collections::HashMap;
use std::path::Path;
use tokio::sync::RwLock;
use crate::error::{I18nError, Result};
use crate::locale::{Locale, LocaleManager};
use crate::loader::{TranslationLoader, TranslationMap, FileSystemLoader};
use crate::interpolator::{Interpolator, InterpolationContext};
use crate::formatter::Formatter;

/// Main translator struct that provides the public API
pub struct Translator {
    /// Current locale
    current_locale: RwLock<Locale>,
    /// Fallback locale
    fallback_locale: Locale,
    /// Translation loader
    pub loader: RwLock<Box<dyn TranslationLoader + Send + Sync>>,
    /// Interpolator for variables
    interpolator: Interpolator,
    /// Formatter for localized formatting
    formatter: Formatter,
    /// Locale manager
    locale_manager: LocaleManager,
    /// Cache of valid translations per locale (for error recovery)
    translation_cache: RwLock<HashMap<String, TranslationMap>>,
}

impl Clone for Translator {
    fn clone(&self) -> Self {
        // We need to clone the loader, but since it's a trait object, we recreate it
        // with the same base directory. This is a limitation of the current design.
        // In a real implementation, the loader should be clonable or we should use
        // Arc for the loader itself.
        let loader_base_dir = "translations"; // Default for cloned instances
        // TODO: Make loader clonable to preserve the original base directory
        
        Self {
            current_locale: RwLock::new(self.current_locale.try_read().unwrap().clone()),
            fallback_locale: self.fallback_locale.clone(),
            loader: RwLock::new(Box::new(FileSystemLoader::new(loader_base_dir))),
            interpolator: self.interpolator.clone(),
            formatter: self.formatter.clone(),
            locale_manager: self.locale_manager.clone(),
            translation_cache: RwLock::new(HashMap::new()), // Start with empty cache for clones
        }
    }
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
            loader: RwLock::new(Box::new(FileSystemLoader::new("translations"))),
            interpolator: Interpolator::new().unwrap(),
            formatter: Formatter::new(),
            locale_manager: LocaleManager::new(),
            translation_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Set the current locale
    pub async fn set_locale(&self, locale: Locale) -> Result<()> {
        {
            let mut current = self.current_locale.write().await;
            *current = locale;
        }
        // Reload translations for the new locale
        self.reload_translations().await
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
        let dir = dir.as_ref();
        let loader = self.loader.read().await;
        
        // Try to load common locales
        let locales = ["en", "es", "pt", "fr", "de", "it", "ja", "zh", "ru"];
        
        for locale_str in &locales {
            if let Ok(locale) = Locale::from(*locale_str) {
                // Try to load translations for this locale - ignore errors for missing files
                let _ = loader.load_translations(locale_str).await;
                // If successful, store in cache
                if let Ok(translations) = loader.load_translations(locale_str).await {
                    let mut cache = self.translation_cache.write().await;
                    cache.insert(locale_str.to_string(), translations);
                }
            }
        }
        
        Ok(())
    }

    /// Load translations for a specific locale
    pub async fn load_translations_for_locale(&self, locale: &Locale) -> Result<()> {
        let loader = self.loader.read().await;
        let translations = loader.load_translations(locale.as_str()).await?;
        
        // Store in cache
        let mut cache = self.translation_cache.write().await;
        cache.insert(locale.as_str().to_string(), translations);
        
        Ok(())
    }

    /// Translate a key with optional variables
    pub async fn translate(&mut self, key: &str, variables: &[(&str, &str)]) -> Result<String> {
        let current_locale = self.current_locale.read().await.clone();
        let variables_map = variables.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<HashMap<_, _>>();
        self.translate_with_locale(key, &current_locale, Some(&variables_map)).await
    }

    /// Translate a key for a specific locale
    pub async fn translate_with_locale(&mut self, key: &str, locale: &Locale, variables: Option<&HashMap<String, String>>) -> Result<String> {
        let cache = self.translation_cache.read().await;

        // Try the requested locale first (from cache)
        if let Some(translations) = cache.get(locale.as_str()) {
            println!("DEBUG: Using cached translations for {}: {:?}", locale, translations);
            if let Some(value) = self.get_nested_value(translations, key) {
                println!("DEBUG: Found value for key '{}': {}", key, value);
                let interpolated = {
                    let mut context = InterpolationContext::new(locale);
                    // Add passed variables
                    if let Some(vars) = variables {
                        for (k, v) in vars.iter() {
                            context.variables.insert(k.clone(), v.clone());
                        }
                    }
                    // Add translations for recursive interpolation
                    self.add_translations_to_context(translations, &mut context.variables, "");
                    self.interpolator.interpolate(&value, &context)?
                };
                return Ok(interpolated);
            } else {
                println!("DEBUG: Key '{}' not found in cached translations for {}", key, locale);
            }
        } else {
            println!("DEBUG: No cached translations found for {}", locale);
        }

        // Try configured fallback locale (from cache)
        if let Some(translations) = cache.get(self.fallback_locale.as_str()) {
            println!("DEBUG: Using cached configured fallback translations for {}: {:?}", self.fallback_locale, translations);
            if let Some(value) = self.get_nested_value(translations, key) {
                println!("DEBUG: Found value for key '{}' in configured fallback: {}", key, value);
                let interpolated = {
                    let mut context = InterpolationContext::new(&self.fallback_locale);
                    // Add passed variables
                    if let Some(vars) = variables {
                        for (k, v) in vars.iter() {
                            context.variables.insert(k.clone(), v.clone());
                        }
                    }
                    // Add translations for recursive interpolation
                    self.add_translations_to_context(translations, &mut context.variables, "");
                    self.interpolator.interpolate(&value, &context)?
                };
                return Ok(interpolated);
            }
        }

        // Try locale fallback chain
        let fallback_chain = locale.fallback_chain();
        for fallback_locale in fallback_chain {
            if let Some(translations) = cache.get(fallback_locale.as_str()) {
                println!("DEBUG: Using cached fallback translations for {}: {:?}", fallback_locale, translations);
                if let Some(value) = self.get_nested_value(translations, key) {
                    println!("DEBUG: Found value for key '{}' in fallback {}: {}", key, fallback_locale, value);
                    let interpolated = {
                        let mut context = InterpolationContext::new(&fallback_locale);
                        // Add passed variables
                        if let Some(vars) = variables {
                            for (k, v) in vars.iter() {
                                context.variables.insert(k.clone(), v.clone());
                            }
                        }
                        // Add translations for recursive interpolation
                        self.add_translations_to_context(translations, &mut context.variables, "");
                        self.interpolator.interpolate(&value, &context)?
                    };
                    return Ok(interpolated);
                }
            }
        }

        // If we get here, the key was not found in any locale
        Err(I18nError::TranslationNotFound { key: key.to_string() })
    }

    fn add_translations_to_context(&self, translations: &TranslationMap, context: &mut HashMap<String, String>, prefix: &str) {
        for (key, value) in translations.iter() {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            
            match value {
                serde_json::Value::String(s) => {
                    context.insert(full_key, s.clone());
                }
                serde_json::Value::Object(obj) => {
                    self.add_translations_to_context(&serde_json::Map::from_iter(obj.clone()), context, &full_key);
                }
                _ => {
                    context.insert(full_key, value.to_string());
                }
            }
        }
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
        if let Ok(translations) = loader.load_translations(locale.as_str()).await {
            if self.get_nested_value(&translations, key).is_some() {
                return true;
            }
        }

        // Try fallback locales
        for fallback_locale in locale.fallback_chain() {
            if let Ok(translations) = loader.load_translations(fallback_locale.as_str()).await {
                if self.get_nested_value(&translations, key).is_some() {
                    return true;
                }
            }
        }

        false
    }

    /// Get all available locales
    pub async fn available_locales(&self) -> Vec<Locale> {
        // TODO: Implement proper locale discovery
        vec![]
    }

    /// Clear the translation cache
    pub async fn clear_cache(&self) {
        // TODO: Implement cache clearing
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
        // First try to get the exact key (handles flattened keys like "welcome.message")
        if let Some(value) = translations.get(key) {
            match value {
                serde_json::Value::String(s) => return Some(s.clone()),
                _ => return None,
            }
        }

        // If not found, try navigating nested objects
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 1 {
            return None; // Already tried the exact key above
        }

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
    pub async fn translate_plural(&mut self, key: &str, count: i64, variables: &[(&str, &str)]) -> Result<String> {
        // Read current locale first to avoid borrowing issues
        let current_locale_str = self.current_locale.read().await.clone();

        // Add count to variables
        let count_str = count.to_string();
        let mut all_vars = vec![("count", count_str.as_str())];
        all_vars.extend_from_slice(variables);

        self.translate(key, &all_vars).await
    }

    /// Get translation with fallback to a default value
    pub async fn translate_or(&mut self, key: &str, default: &str, variables: &[(&str, &str)]) -> String {
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

    /// Clear cache and reload all translations
    pub async fn reload_translations(&self) -> Result<()> {
        let loader = self.loader.read().await;
        loader.clear_cache().await;

        let current_locale = self.current_locale.read().await.clone();

        // Try to reload current locale translations
        match loader.load_translations(current_locale.as_str()).await {
            Ok(new_translations) => {
                // Success: update cache
                let mut cache = self.translation_cache.write().await;
                cache.insert(current_locale.as_str().to_string(), new_translations);
                println!("DEBUG: Successfully reloaded translations for {}", current_locale);
            }
            Err(e) => {
                // Failure: keep existing cached translations
                println!("DEBUG: Failed to reload translations for {}: {:?}, keeping cached versions", current_locale, e);
            }
        }

        // Try to reload fallback locale translations
        match loader.load_translations(self.fallback_locale.as_str()).await {
            Ok(new_translations) => {
                let mut cache = self.translation_cache.write().await;
                cache.insert(self.fallback_locale.as_str().to_string(), new_translations);
                println!("DEBUG: Successfully reloaded fallback translations for {}", self.fallback_locale);
            }
            Err(e) => {
                println!("DEBUG: Failed to reload fallback translations for {}: {:?}, keeping cached versions", self.fallback_locale, e);
            }
        }

        Ok(())
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
        let loader = Box::new(FileSystemLoader::new(&self.translations_dir));
        println!("DEBUG: TranslatorBuilder created loader with base_dir: {}", self.translations_dir);
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
        let temp_path = temp_dir.into_path(); // Convert to PathBuf to prevent auto-deletion

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

        write(temp_path.join("en.json"), en_translations).await.unwrap();
        write(temp_path.join("es.json"), es_translations).await.unwrap();

        let translator = TranslatorBuilder::new()
            .with_locale(Locale::from("en").unwrap())
            .with_translations_dir(temp_path.to_string_lossy().to_string())
            .build();

        println!("DEBUG: Translator created with translations dir: {}", temp_path.display());

        translator.load_translations_from_dir(&temp_path).await.unwrap();

        translator
    }

    #[tokio::test]
    async fn test_simple_translation() {
        let mut translator = create_test_translator().await;

        let result = translator.translate("greeting.hello", &[]).await.unwrap();
        assert_eq!(result, "Hello");
    }

    #[tokio::test]
    async fn test_translation_with_variables() {
        let mut translator = create_test_translator().await;

        let result = translator.translate("messages.welcome", &[("name", "Alice")]).await.unwrap();
        assert_eq!(result, "Welcome, Alice!");
    }

    #[tokio::test]
    async fn test_plural_translation() {
        let mut translator = create_test_translator().await;

        let result = translator.translate("messages.items", &[("count", "1")]).await.unwrap();
        assert_eq!(result, "You have # item");

        let result = translator.translate("messages.items", &[("count", "5")]).await.unwrap();
        assert_eq!(result, "You have # items");
    }

    #[tokio::test]
    async fn test_locale_switching() {
        let mut translator = create_test_translator().await;

        // English
        let result = translator.translate("greeting.hello", &[]).await.unwrap();
        assert_eq!(result, "Hello");

        // Switch to Spanish
        translator.set_locale(Locale::from("es").unwrap()).await.unwrap();
        let result = translator.translate("greeting.hello", &[]).await.unwrap();
        assert_eq!(result, "Hola");
    }

    #[tokio::test]
    async fn test_missing_key() {
        let mut translator = create_test_translator().await;

        let result = translator.translate("nonexistent.key", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_translate_or_fallback() {
        let mut translator = create_test_translator().await;

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
