/*!
Decorator system for i18n classes and hot reload functionality.
*/

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use crate::error::{I18nError, Result};
use crate::locale::Locale;
use crate::translator::Translator;

/// I18n decorator for classes that need translation capabilities
pub struct I18nDecorator {
    /// Translator instance
    translator: Arc<RwLock<Translator>>,
    /// Hot reload watcher
    hot_reload: Option<HotReloadManager>,
    /// Decorated classes registry
    decorated_classes: RwLock<HashMap<String, DecoratedClassInfo>>,
}

/// Information about a decorated class
#[derive(Clone)]
pub struct DecoratedClassInfo {
    /// Class name
    pub class_name: String,
    /// Locale for this class
    pub locale: Locale,
    /// Fallback locale
    pub fallback_locale: Option<Locale>,
    /// Hot reload enabled
    pub hot_reload: bool,
    /// Translation keys used by this class
    pub translation_keys: Vec<String>,
}

impl I18nDecorator {
    /// Create a new i18n decorator
    pub fn new(translator: Arc<RwLock<Translator>>) -> Self {
        Self {
            translator,
            hot_reload: None,
            decorated_classes: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new i18n decorator with hot reload
    pub async fn with_hot_reload(translator: Arc<RwLock<Translator>>, watch_paths: &[&Path]) -> Result<Self> {
        let hot_reload = HotReloadManager::new(watch_paths).await?;

        let decorator = Self {
            translator: translator.clone(),
            hot_reload: Some(hot_reload),
            decorated_classes: RwLock::new(HashMap::new()),
        };

        // Set up hot reload callback to reload translations
        if let Some(ref hr) = decorator.hot_reload {
            let translator_clone = decorator.translator.clone();
            hr.set_on_change(move || {
                let translator = translator_clone.clone();
                tokio::spawn(async move {
                    let mut translator = translator.write().await;
                    if let Err(e) = translator.reload_translations().await {
                        eprintln!("Failed to reload translations: {:?}", e);
                    }
                });
            }).await;
        }

        Ok(decorator)
    }

    /// Decorate a class with i18n capabilities
    pub async fn decorate_class(
        &self,
        class_name: &str,
        locale: Locale,
        fallback_locale: Option<Locale>,
        hot_reload: bool,
        translation_keys: Vec<String>,
    ) -> Result<()> {
        let info = DecoratedClassInfo {
            class_name: class_name.to_string(),
            locale,
            fallback_locale,
            hot_reload,
            translation_keys,
        };

        let mut classes = self.decorated_classes.write().await;
        classes.insert(class_name.to_string(), info);

        Ok(())
    }

    /// Get decorated class information
    pub async fn get_class_info(&self, class_name: &str) -> Option<DecoratedClassInfo> {
        let classes = self.decorated_classes.read().await;
        classes.get(class_name).cloned()
    }

    /// Translate for a specific decorated class
    pub async fn translate_for_class(
        &self,
        class_name: &str,
        key: &str,
        variables: &[(&str, &str)],
    ) -> Result<String> {
        let classes = self.decorated_classes.read().await;

        if let Some(class_info) = classes.get(class_name) {
            // Check if the key is registered for this class
            if !class_info.translation_keys.contains(&key.to_string()) {
                return Err(I18nError::translation_not_found(format!(
                    "Key '{}' not registered for class '{}'",
                    key, class_name
                )));
            }

            // Temporarily switch locale for this translation
            let mut translator = self.translator.write().await;
            let original_locale = translator.get_locale().await;
            translator.set_locale(class_info.locale.clone()).await;

            let result = translator.translate(key, variables).await;

            // Restore original locale
            translator.set_locale(original_locale).await;

            result
        } else {
            Err(I18nError::translation_not_found(format!(
                "Class '{}' not decorated with @i18n",
                class_name
            )))
        }
    }

    /// Get all decorated classes
    pub async fn get_decorated_classes(&self) -> Vec<DecoratedClassInfo> {
        let classes = self.decorated_classes.read().await;
        classes.values().cloned().collect()
    }

    /// Check if hot reload is enabled
    pub fn has_hot_reload(&self) -> bool {
        self.hot_reload.is_some()
    }

    /// Start hot reload monitoring
    pub async fn start_hot_reload(&mut self) -> Result<()> {
        if let Some(ref mut hot_reload) = self.hot_reload {
            hot_reload.start().await?;
        }
        Ok(())
    }

    /// Stop hot reload monitoring
    pub async fn stop_hot_reload(&mut self) -> Result<()> {
        if let Some(ref hot_reload) = self.hot_reload {
            hot_reload.stop().await?;
        }
        Ok(())
    }

    /// Get the underlying translator
    pub fn translator(&self) -> &Arc<RwLock<Translator>> {
        &self.translator
    }

    /// Get a mutable reference to the translator
    pub fn translator_mut(&mut self) -> &mut Arc<RwLock<Translator>> {
        &mut self.translator
    }
}

/// Hot reload manager for translation files
pub struct HotReloadManager {
    /// File watcher
    _watcher: RecommendedWatcher,
    /// Paths being watched
    watch_paths: Vec<std::path::PathBuf>,
    /// Debounce duration to avoid excessive reloads
    debounce_duration: Duration,
    /// Last reload time for debouncing
    last_reload: Arc<RwLock<Instant>>,
    /// Channel for file change events
    event_tx: Sender<notify::Event>,
    /// Channel receiver for processing events
    event_rx: Arc<RwLock<Option<Receiver<notify::Event>>>>,
    /// Callback for when files change
    on_change: Arc<RwLock<Option<Box<dyn Fn() + Send + Sync>>>>,
    /// Whether hot reload is active
    is_active: Arc<RwLock<bool>>,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    pub async fn new(watch_paths: &[&Path]) -> Result<Self> {
        let (tx, rx) = channel(100);
        let tx_clone = tx.clone();

        let watcher = RecommendedWatcher::new(
            move |res: std::result::Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Send event through channel for async processing
                        let _ = tx_clone.send(event);
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            },
            Config::default(),
        ).map_err(|e| I18nError::IoError {
            message: format!("Failed to create file watcher: {}", e),
            source: std::io::Error::new(std::io::ErrorKind::Other, "notify error"),
        })?;

        Ok(Self {
            _watcher: watcher,
            watch_paths: watch_paths.iter().map(|p| p.to_path_buf()).collect(),
            debounce_duration: Duration::from_millis(300), // 300ms debounce
            last_reload: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(1))),
            event_tx: tx,
            event_rx: Arc::new(RwLock::new(Some(rx))),
            on_change: Arc::new(RwLock::new(None)),
            is_active: Arc::new(RwLock::new(false)),
        })
    }

    /// Set the change callback
    pub async fn set_on_change<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut on_change = self.on_change.write().await;
        *on_change = Some(Box::new(callback));
    }

    /// Start watching for file changes
    pub async fn start(&mut self) -> Result<()> {
        let mut is_active = self.is_active.write().await;
        if *is_active {
            return Ok(()); // Already started
        }

        // Watch all paths
        for path in &self.watch_paths {
            self._watcher.watch(path, RecursiveMode::Recursive)
                .map_err(|e| I18nError::IoError {
                    message: format!("Failed to watch path: {}", e),
                    source: std::io::Error::new(std::io::ErrorKind::Other, "notify error"),
                })?;
        }

        *is_active = true;

        // Start event processing task
        let event_rx = self.event_rx.write().await.take();
        if let Some(mut rx) = event_rx {
            let on_change = self.on_change.clone();
            let last_reload = self.last_reload.clone();
            let debounce_duration = self.debounce_duration;
            let is_active = self.is_active.clone();

            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    // Check if still active
                    if !*is_active.read().await {
                        break;
                    }

                    // Check if event is relevant (JSON/YAML files)
                    if Self::is_relevant_event(&event) {
                        // Debounce: check time since last reload
                        let now = Instant::now();
                        let last = *last_reload.read().await;
                        if now.duration_since(last) >= debounce_duration {
                            // Update last reload time
                            *last_reload.write().await = now;

                            // Call callback if set
                            if let Some(ref callback) = *on_change.read().await {
                                callback();
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Stop watching for file changes
    pub async fn stop(&self) -> Result<()> {
        let mut is_active = self.is_active.write().await;
        *is_active = false;

        // The watcher will be dropped when HotReloadManager is dropped
        Ok(())
    }

    /// Check if files have changed (for polling-based implementations)
    pub async fn check_for_changes(&self) -> Result<bool> {
        // With async file watching, this is not needed
        // Changes are detected automatically
        Ok(false)
    }

    /// Check if an event is relevant for translation reloading
    fn is_relevant_event(event: &notify::Event) -> bool {
        // Only care about JSON and YAML files
        event.paths.iter().any(|path| {
            if let Some(ext) = path.extension() {
                ext == "json" || ext == "yaml" || ext == "yml"
            } else {
                false
            }
        }) && matches!(
            event.kind,
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) | notify::EventKind::Remove(_)
        )
    }

    /// Set debounce duration
    pub fn with_debounce_duration(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }
}

/// Macro-like function to create an i18n-decorated class
/// This simulates what a real @i18n decorator would do at compile time
pub fn create_i18n_class<F>(
    class_name: &str,
    locale: Locale,
    fallback_locale: Option<Locale>,
    hot_reload: bool,
    translation_keys: Vec<String>,
    class_factory: F,
) -> Result<I18nClassWrapper>
where
    F: FnOnce() -> Box<dyn I18nClass>,
{
    Ok(I18nClassWrapper {
        class_name: class_name.to_string(),
        locale,
        fallback_locale,
        hot_reload,
        translation_keys,
        instance: class_factory(),
    })
}

/// Trait for i18n-enabled classes
pub trait I18nClass: Send + Sync {
    /// Get the class name
    fn class_name(&self) -> &str;

    /// Get translation keys used by this class
    fn translation_keys(&self) -> &[String];

    /// Translate a key for this class
    fn translate<'a>(&'a self, decorator: &'a I18nDecorator, key: String, variables: Vec<(String, String)>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send + 'a>>;
}

/// Wrapper for i18n-decorated classes
pub struct I18nClassWrapper {
    pub class_name: String,
    pub locale: Locale,
    pub fallback_locale: Option<Locale>,
    pub hot_reload: bool,
    pub translation_keys: Vec<String>,
    pub instance: Box<dyn I18nClass>,
}

impl I18nClassWrapper {
    /// Register this class with a decorator
    pub async fn register_with(&self, decorator: &I18nDecorator) -> Result<()> {
        decorator.decorate_class(
            &self.class_name,
            self.locale.clone(),
            self.fallback_locale.clone(),
            self.hot_reload,
            self.translation_keys.clone(),
        ).await
    }
}

/// Example implementation of an i18n-enabled class
#[derive(Debug)]
pub struct MessageService {
    class_name: String,
    translation_keys: Vec<String>,
}

impl MessageService {
    pub fn new() -> Self {
        Self {
            class_name: "MessageService".to_string(),
            translation_keys: vec![
                "welcome.message".to_string(),
                "error.not_found".to_string(),
                "success.saved".to_string(),
            ],
        }
    }
}

impl I18nClass for MessageService {
    fn class_name(&self) -> &str {
        &self.class_name
    }

    fn translation_keys(&self) -> &[String] {
        &self.translation_keys
    }

    fn translate<'a>(&'a self, decorator: &'a I18nDecorator, key: String, variables: Vec<(String, String)>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send + 'a>> {
        let class_name = self.class_name.to_string();

        Box::pin(async move {
            decorator.translate_for_class(&class_name, &key, &variables.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>()).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator::TranslatorBuilder;
    use tempfile::tempdir;
    use tokio::fs::write;

    async fn create_test_decorator() -> I18nDecorator {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.into_path(); // Convert to PathBuf to prevent auto-deletion

        // Create test translation files
        let en_translations = r#"{
            "welcome.message": "Welcome to our app!",
            "error": {
                "not_found": "Item not found"
            }
        }"#;

        write(temp_path.join("en.json"), en_translations).await.unwrap();

        let translator = TranslatorBuilder::new()
            .with_locale(Locale::from("en").unwrap())
            .with_translations_dir(temp_path.to_string_lossy().to_string())
            .build();

        translator.load_translations_from_dir(&temp_path).await.unwrap();

        I18nDecorator::new(Arc::new(RwLock::new(translator)))
    }

    #[tokio::test]
    async fn test_decorate_class() {
        let decorator = create_test_decorator().await;

        let translation_keys = vec![
            "welcome.message".to_string(),
            "error.not_found".to_string(),
        ];

        decorator.decorate_class(
            "TestService",
            Locale::from("en").unwrap(),
            None,
            false,
            translation_keys,
        ).await.unwrap();

        let class_info = decorator.get_class_info("TestService").await.unwrap();
        assert_eq!(class_info.class_name, "TestService");
        assert_eq!(class_info.translation_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_translate_for_class() {
        let decorator = create_test_decorator().await;

        let translation_keys = vec!["welcome.message".to_string()];
        decorator.decorate_class(
            "TestService",
            Locale::from("en").unwrap(),
            None,
            false,
            translation_keys,
        ).await.unwrap();

        let result = decorator.translate_for_class(
            "TestService",
            "welcome.message",
            &[],
        ).await.unwrap();

        assert_eq!(result, "Welcome to our app!");
    }

    #[tokio::test]
    async fn test_translate_unregistered_key() {
        let decorator = create_test_decorator().await;

        decorator.decorate_class(
            "TestService",
            Locale::from("en").unwrap(),
            None,
            false,
            vec!["welcome.message".to_string()],
        ).await.unwrap();

        let result = decorator.translate_for_class(
            "TestService",
            "error.not_found",
            &[],
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_undecorated_class() {
        let decorator = create_test_decorator().await;

        let result = decorator.translate_for_class(
            "NonExistentService",
            "welcome.message",
            &[],
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_i18n_class_wrapper() {
        let decorator = create_test_decorator().await;

        let wrapper = create_i18n_class(
            "MessageService",
            Locale::from("en").unwrap(),
            None,
            false,
            vec!["welcome.message".to_string()],
            || Box::new(MessageService::new()),
        ).unwrap();

        wrapper.register_with(&decorator).await.unwrap();

        let class_info = decorator.get_class_info("MessageService").await.unwrap();
        assert_eq!(class_info.class_name, "MessageService");
    }
}