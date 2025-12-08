/*!
# Comprehensive Tests for Vela i18n System

This module contains all comprehensive tests for the i18n system including
integration tests, performance benchmarks, concurrency tests, and error recovery tests.
*/

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::decorator::I18nDecorator;
    use crate::locale::Locale;
    use crate::translator::{Translator, TranslatorBuilder};
    use crate::loader::FileSystemLoader;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::fs::write;

    // Mock service for testing
    struct MessageService {
        pub base_url: String,
    }

    impl MessageService {
        fn new() -> Self {
            Self {
                base_url: "https://api.example.com".to_string(),
            }
        }

        async fn get_welcome_message(&self, user_name: &str) -> String {
            format!("Welcome, {}!", user_name)
        }
    }

    /// Test the complete i18n workflow from file to translation
    #[tokio::test]
    async fn test_full_i18n_workflow() {
        // Setup temporary directory with translation files
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create English translations
        let en_translations = r#"{
            "app": {
                "name": "Vela App",
                "version": "1.0.0"
            },
            "messages": {
                "welcome": "Welcome to Vela App!",
                "greeting": "Hello, {{name}}!",
                "error": {
                    "not_found": "Item not found",
                    "server_error": "Internal server error"
                }
            },
            "buttons": {
                "save": "Save",
                "cancel": "Cancel",
                "delete": "Delete"
            }
        }"#;

        // Create Spanish translations
        let es_translations = r#"{
            "app": {
                "name": "Aplicación Vela",
                "version": "1.0.0"
            },
            "messages": {
                "welcome": "¡Bienvenido a Aplicación Vela!",
                "greeting": "¡Hola, {{name}}!",
                "error": {
                    "not_found": "Elemento no encontrado",
                    "server_error": "Error interno del servidor"
                }
            },
            "buttons": {
                "save": "Guardar",
                "cancel": "Cancelar",
                "delete": "Eliminar"
            }
        }"#;

        // Write translation files
        write(temp_path.join("en.json"), en_translations).await.unwrap();
        write(temp_path.join("es.json"), es_translations).await.unwrap();

        // Create translator with English as primary locale
        let translator = Arc::new(tokio::sync::RwLock::new(
            TranslatorBuilder::new()
                .with_locale(Locale::from("en").unwrap())
                .with_fallback_locale(Locale::from("es").unwrap())
                .with_translations_dir(temp_path.to_string_lossy().to_string())
                .build()
        ));

        println!("DEBUG: Configured translations dir: {}", temp_path.to_string_lossy());

        // Load translations
        translator.read().await.load_translations_from_dir(temp_path).await.unwrap();

        // Create decorator
        let decorator = I18nDecorator::new(translator.clone());

        // Register a service class with translations
        let translation_keys = vec![
            "messages.welcome".to_string(),
            "messages.greeting".to_string(),
            "buttons.save".to_string(),
            "buttons.cancel".to_string(),
        ];

        decorator.decorate_class(
            "UIService",
            Locale::from("en").unwrap(),
            Some(Locale::from("es").unwrap()),
            false,
            translation_keys,
        ).await.unwrap();

        // Test basic translations
        let welcome = decorator.translate_for_class(
            "UIService",
            "messages.welcome",
            &[],
        ).await.unwrap();
        assert_eq!(welcome, "Welcome to Vela App!");

        let greeting = decorator.translate_for_class(
            "UIService",
            "messages.greeting",
            &[("name", "Alice")],
        ).await.unwrap();
        assert_eq!(greeting, "Hello, Alice!");

        // Test fallback to Spanish
        let welcome_fallback = translator.write().await.translate_with_locale("messages.welcome", &Locale::from("fr").unwrap(), None).await.unwrap();
        assert_eq!(welcome_fallback, "¡Bienvenido a Aplicación Vela!"); // Falls back to Spanish

        // Reset to English
        translator.write().await.set_locale(Locale::from("en").unwrap()).await.unwrap();
    }

    /// Test hot reload integration with file changes
    #[tokio::test]
    async fn test_hot_reload_integration() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create translator WITHOUT fallback to avoid loading from default "translations" directory
        let temp_loader = Box::new(FileSystemLoader::new(&temp_path.to_string_lossy().to_string()));
        let translator = Arc::new(tokio::sync::RwLock::new({
            let mut t = Translator::with_locale(Locale::from("en").unwrap());
            t.loader = tokio::sync::RwLock::new(temp_loader);
            // Set fallback to the same locale to prevent any fallback loading
            t.set_fallback_locale(Locale::from("en").unwrap());
            t
        }));

        let decorator = I18nDecorator::with_hot_reload(
            Arc::clone(&translator),
            &[&temp_path],
        ).await.unwrap();

        // Create initial translation file AFTER creating translator
        let initial_translations = r#"{
            "message": "Initial message"
        }"#;

        write(temp_path.join("en.json"), initial_translations).await.unwrap();

        // Register class
        decorator.decorate_class(
            "TestService",
            Locale::from("en").unwrap(),
            None,
            false,
            vec!["message".to_string()],
        ).await.unwrap();

        // Test initial translation
        let initial = decorator.translate_for_class(
            "TestService",
            "message",
            &[],
        ).await.unwrap();
        assert_eq!(initial, "Initial message");

        // Modify translation file
        let updated_translations = r#"{
            "message": "Updated message"
        }"#;

        write(temp_path.join("en.json"), updated_translations).await.unwrap();

        // Wait for hot reload (with timeout)
        let result = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let translation = decorator.translate_for_class(
                    "TestService",
                    "message",
                    &[],
                ).await.unwrap();

                if translation == "Updated message" {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }).await;

        assert!(result.is_ok(), "Hot reload should have updated translation within 2 seconds");
    }
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use crate::decorator::I18nDecorator;
    use crate::locale::Locale;
    use crate::translator::{Translator, TranslatorBuilder};
    use crate::loader::FileSystemLoader;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::fs::write;

    /// Test thread safety of decorator operations
    #[tokio::test]
    async fn test_thread_safety() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create translations
        let translations = r#"{
            "message": "Hello World",
            "greeting": "Hi {{name}}"
        }"#;

        write(temp_path.join("en.json"), translations).await.unwrap();

        let translator = TranslatorBuilder::new()
            .with_locale(Locale::from("en").unwrap())
            .with_translations_dir(temp_path.to_string_lossy().to_string())
            .build();

        translator.load_translations_from_dir(temp_path).await.unwrap();

        let decorator = Arc::new(I18nDecorator::new(Arc::new(tokio::sync::RwLock::new(translator))));

        // Spawn multiple tasks that access the decorator concurrently
        let mut handles = Vec::new();

        for i in 0..10 {
            let decorator_clone: Arc<I18nDecorator> = Arc::clone(&decorator);
            let handle = tokio::spawn(async move {
                // Each task registers its own class
                let class_name = format!("Service{}", i);
                decorator_clone.decorate_class(
                    &class_name,
                    Locale::from("en").unwrap(),
                    None,
                    false,
                    vec!["message".to_string(), "greeting".to_string()],
                ).await.unwrap();

                // Perform translations
                for _ in 0..100 {
                    let _ = decorator_clone.translate_for_class(
                        &class_name,
                        "message",
                        &[],
                    ).await.unwrap();

                    let _ = decorator_clone.translate_for_class(
                        &class_name,
                        "greeting",
                        &[("name", "Test")],
                    ).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all classes were registered correctly
        for i in 0..10 {
            let class_name = format!("Service{}", i);
            let class_info = decorator.get_class_info(&class_name).await.unwrap();
            assert_eq!(class_info.class_name, class_name);
            assert_eq!(class_info.translation_keys.len(), 2);
        }
    }
}

#[cfg(test)]
mod error_recovery_tests {
    use super::*;
    use crate::decorator::I18nDecorator;
    use crate::locale::Locale;
    use crate::translator::{Translator, TranslatorBuilder};
    use crate::loader::FileSystemLoader;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::fs::{write, remove_file};

    /// Test recovery from corrupt JSON files
    #[tokio::test]
    async fn test_corrupt_file_recovery() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        // Create translator WITHOUT fallback to avoid loading from default "translations" directory
        let temp_loader = Box::new(FileSystemLoader::new(&temp_path.to_string_lossy().to_string()));
        let translator = Arc::new(tokio::sync::RwLock::new({
            let mut t = Translator::with_locale(Locale::from("en").unwrap());
            t.loader = tokio::sync::RwLock::new(temp_loader);
            // Set fallback to the same locale to prevent any fallback loading
            t.set_fallback_locale(Locale::from("en").unwrap());
            t
        }));

        let decorator = I18nDecorator::with_hot_reload(
            Arc::clone(&translator),
            &[&temp_path],
        ).await.unwrap();

        // Create valid initial translations AFTER creating translator
        let valid_translations = r#"{
            "message": "Valid message",
            "error": "Valid error"
        }"#;

        write(temp_path.join("en.json"), valid_translations).await.unwrap();

        // Force reload after creating the file
        translator.write().await.reload_translations().await.unwrap();

        // Register class
        decorator.decorate_class(
            "TestService",
            Locale::from("en").unwrap(),
            None,
            false,
            vec!["message".to_string(), "error".to_string()],
        ).await.unwrap();

        // Test initial valid translations
        let message = decorator.translate_for_class(
            "TestService",
            "message",
            &[],
        ).await.unwrap();
        assert_eq!(message, "Valid message");

        // Write corrupt JSON (missing closing brace and invalid syntax)
        let corrupt_translations = r#"{
            "message": "Corrupt message",
            "error": "Corrupt error"
            "missing": "comma
        }"#;

        write(temp_path.join("en.json"), corrupt_translations).await.unwrap();

        // Wait for file watching to detect change
        tokio::time::sleep(Duration::from_millis(500)).await;

        // System should maintain old valid translations
        let message_after_corrupt = decorator.translate_for_class(
            "TestService",
            "message",
            &[],
        ).await.unwrap();
        assert_eq!(message_after_corrupt, "Valid message");

        // Fix the file
        let fixed_translations = r#"{
            "message": "Fixed message",
            "error": "Fixed error"
        }"#;

        write(temp_path.join("en.json"), fixed_translations).await.unwrap();

        // Wait for recovery
        let result = tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let translation = decorator.translate_for_class(
                    "TestService",
                    "message",
                    &[],
                ).await.unwrap();

                if translation == "Fixed message" {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }).await;

        assert!(result.is_ok(), "Should recover from corrupt file within 3 seconds");

        // Verify both translations work after recovery
        let fixed_message = decorator.translate_for_class(
            "TestService",
            "message",
            &[],
        ).await.unwrap();
        assert_eq!(fixed_message, "Fixed message");

        let fixed_error = decorator.translate_for_class(
            "TestService",
            "error",
            &[],
        ).await.unwrap();
        assert_eq!(fixed_error, "Fixed error");
    }
}