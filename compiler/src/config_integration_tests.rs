"""
Tests de Integración para Config Management

Jira: VELA-609
Historia: VELA-609
"""

use std::fs;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use tokio::time::sleep;
use crate::config_loader::{ConfigLoader, ConfigSource, ConfigError, RequiredValidator, RangeValidator, EmailValidator};
use crate::hot_reload::{HotReloadManager, HotReloadBuilder, ConfigChangeEvent, ReloadState};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_config_loading_priority() {
        // Crear archivos temporales
        let file_config = r#"{
            "app.name": "file_app",
            "app.version": "1.0.0",
            "database.host": "localhost",
            "database.port": 5432
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, file_config).unwrap();

        // Set env vars (higher priority)
        env::set_var("VELA_APP_NAME", "env_app");
        env::set_var("VELA_DATABASE_HOST", "prod-db.example.com");

        let mut loader = ConfigLoader::new()
            .add_source(ConfigSource::File(temp_file.path().to_str().unwrap().to_string()))
            .add_validator("app.name".to_string(), RequiredValidator)
            .add_validator("database.port".to_string(), RangeValidator { min: Some(1024), max: Some(65535) });

        // Load config
        loader.load().unwrap();

        // Verify priority: env vars override file
        assert_eq!(loader.get_string("app.name"), Some("env_app".to_string())); // from env
        assert_eq!(loader.get_string("app.version"), Some("1.0.0".to_string())); // from file
        assert_eq!(loader.get_string("database.host"), Some("prod-db.example.com".to_string())); // from env
        assert_eq!(loader.get_int("database.port").unwrap().unwrap(), 5432); // from file

        // Clean up
        env::remove_var("VELA_APP_NAME");
        env::remove_var("VELA_DATABASE_HOST");
    }

    #[tokio::test]
    async fn test_profile_based_loading() {
        // Config base
        let base_config = r#"{
            "app.name": "myapp",
            "app.debug": true,
            "database.host": "localhost"
        }"#;

        // Config de desarrollo
        let dev_config = r#"{
            "app.debug": false,
            "database.host": "dev-db.local",
            "dev.secret": "dev_key"
        }"#;

        let temp_base = NamedTempFile::new().unwrap();
        let temp_dev = NamedTempFile::new().unwrap();

        fs::write(&temp_base, base_config).unwrap();
        fs::write(&temp_dev, dev_config).unwrap();

        // Simular archivos con nombres de perfil
        fs::write("config.json", base_config).unwrap();
        fs::write("config-dev.json", dev_config).unwrap();

        let mut loader = ConfigLoader::new()
            .with_profile("dev".to_string());

        loader.load().unwrap();

        // Profile config should override base
        assert_eq!(loader.get_string("app.name"), Some("myapp".to_string())); // from base
        assert_eq!(loader.get_bool("app.debug").unwrap().unwrap(), false); // overridden by dev
        assert_eq!(loader.get_string("database.host"), Some("dev-db.local".to_string())); // overridden by dev
        assert_eq!(loader.get_string("dev.secret"), Some("dev_key".to_string())); // only in dev

        // Clean up
        fs::remove_file("config.json").unwrap();
        fs::remove_file("config-dev.json").unwrap();
    }

    #[tokio::test]
    async fn test_validation_integration() {
        let config_data = r#"{
            "user.email": "user@example.com",
            "server.port": 8080,
            "app.name": "test_app"
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config_data).unwrap();

        let mut loader = ConfigLoader::new()
            .add_source(ConfigSource::File(temp_file.path().to_str().unwrap().to_string()))
            .add_validator("user.email".to_string(), EmailValidator)
            .add_validator("server.port".to_string(), RangeValidator { min: Some(1024), max: Some(65535) })
            .add_validator("app.name".to_string(), RequiredValidator);

        // Should load successfully
        let result = loader.load();
        assert!(result.is_ok());

        // Test invalid email
        env::set_var("VELA_USER_EMAIL", "invalid-email");
        let result = loader.load();
        assert!(result.is_err());

        // Test port out of range
        env::set_var("VELA_SERVER_PORT", "80");
        env::remove_var("VELA_USER_EMAIL");
        let result = loader.load();
        assert!(result.is_err());

        // Clean up
        env::remove_var("VELA_USER_EMAIL");
        env::remove_var("VELA_SERVER_PORT");
    }

    #[tokio::test]
    async fn test_hot_reload_end_to_end() {
        // Create initial config
        let initial_config = r#"{"app.version": "1.0.0"}"#;
        fs::write("config.json", initial_config).unwrap();

        let loader = ConfigLoader::new();
        let mut manager = HotReloadBuilder::new()
            .with_loader("test".to_string(), loader).unwrap()
            .with_debounce(Duration::from_millis(100))
            .build()
            .unwrap();

        // Initial load
        manager.force_reload().await.unwrap();

        let loader_ref = manager.get_loader("test").unwrap();
        let initial_version = loader_ref.lock().unwrap().get_string("app.version");
        assert_eq!(initial_version, Some("1.0.0".to_string()));

        // Modify config file
        let updated_config = r#"{"app.version": "2.0.0"}"#;
        fs::write("config.json", updated_config).unwrap();

        // Wait for reload (with debounce)
        sleep(Duration::from_millis(200)).await;

        // Force reload to simulate file change detection
        manager.force_reload().await.unwrap();

        let updated_version = loader_ref.lock().unwrap().get_string("app.version");
        assert_eq!(updated_version, Some("2.0.0".to_string()));

        manager.stop();

        // Clean up
        fs::remove_file("config.json").unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_integration() {
        // Config with invalid JSON
        let invalid_config = r#"{"invalid": json"#;
        fs::write("config.json", invalid_config).unwrap();

        let loader = ConfigLoader::new();
        let result = loader.load();

        // Should fail due to invalid JSON
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Json(_)));

        // Clean up
        fs::remove_file("config.json").unwrap();
    }

    #[tokio::test]
    async fn test_multiple_loaders_coordination() {
        // App config
        let app_config = r#"{"app.name": "myapp", "app.port": 3000}"#;
        fs::write("config.json", app_config).unwrap();

        // DB config
        let db_config = r#"{"db.host": "localhost", "db.port": 5432}"#;
        fs::write("db-config.json", db_config).unwrap();

        let app_loader = ConfigLoader::new();
        let db_loader = ConfigLoader::new()
            .add_source(ConfigSource::File("db-config.json".to_string()));

        let manager = HotReloadBuilder::new()
            .with_loader("app".to_string(), app_loader).unwrap()
            .with_loader("db".to_string(), db_loader).unwrap()
            .build()
            .unwrap();

        // Load all
        manager.force_reload().await.unwrap();

        // Verify both loaders loaded correctly
        let app_loader_ref = manager.get_loader("app").unwrap();
        let db_loader_ref = manager.get_loader("db").unwrap();

        assert_eq!(app_loader_ref.lock().unwrap().get_string("app.name"), Some("myapp".to_string()));
        assert_eq!(db_loader_ref.lock().unwrap().get_string("db.host"), Some("localhost".to_string()));

        // Clean up
        fs::remove_file("config.json").unwrap();
        fs::remove_file("db-config.json").unwrap();
    }

    #[tokio::test]
    async fn test_performance_under_load() {
        // Create a larger config
        let mut large_config = std::collections::HashMap::new();
        for i in 0..1000 {
            large_config.insert(format!("key_{}", i), format!("value_{}", i));
        }

        let json = serde_json::to_string(&large_config).unwrap();
        fs::write("large-config.json", json).unwrap();

        let loader = ConfigLoader::new()
            .add_source(ConfigSource::File("large-config.json".to_string()));

        let start = Instant::now();
        loader.load().unwrap();
        let duration = start.elapsed();

        // Should load 1000 keys in reasonable time (< 100ms)
        assert!(duration < Duration::from_millis(100));

        // Verify some keys
        assert_eq!(loader.get_string("key_0"), Some("value_0".to_string()));
        assert_eq!(loader.get_string("key_999"), Some("value_999".to_string()));

        // Clean up
        fs::remove_file("large-config.json").unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config_data = r#"{"counter": 0}"#;
        fs::write("config.json", config_data).unwrap();

        let loader = Arc::new(Mutex::new(ConfigLoader::new()));
        let loader_clone = loader.clone();

        // Spawn multiple tasks trying to access config
        let handles: Vec<_> = (0..10).map(|i| {
            let loader = loader_clone.clone();
            tokio::spawn(async move {
                let mut loader = loader.lock().unwrap();
                let current = loader.get_int("counter").unwrap().unwrap_or(0);
                // Simulate some work
                sleep(Duration::from_millis(1)).await;
                // In real scenario, this would be atomic update
                current
            })
        }).collect();

        // Wait for all tasks
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        // Clean up
        fs::remove_file("config.json").unwrap();
    }

    #[tokio::test]
    async fn test_hot_reload_with_callbacks() {
        let config_data = r#"{"version": "1.0"}"#;
        fs::write("config.json", config_data).unwrap();

        let callback_called = Arc::new(Mutex::new(false));
        let callback_data = Arc::new(Mutex::new(None));

        let callback_called_clone = callback_called.clone();
        let callback_data_clone = callback_data.clone();

        let manager = HotReloadBuilder::new()
            .with_loader("test".to_string(), ConfigLoader::new()).unwrap()
            .with_callback(move |event| {
                *callback_called_clone.lock().unwrap() = true;
                *callback_data_clone.lock().unwrap() = Some(event.reload_state.clone());
            })
            .build()
            .unwrap();

        // Force reload
        manager.force_reload().await.unwrap();

        // Callback should have been called
        assert!(*callback_called.lock().unwrap());
        let state = callback_data.lock().unwrap().as_ref().unwrap().clone();
        assert_eq!(state, ReloadState::Success);

        // Clean up
        fs::remove_file("config.json").unwrap();
    }

    #[tokio::test]
    async fn test_config_with_env_vars_and_files() {
        // File config
        let file_config = r#"{
            "app.name": "file_app",
            "app.timeout": 30,
            "features": ["auth", "logging"]
        }"#;
        fs::write("config.json", file_config).unwrap();

        // Env vars (higher priority)
        env::set_var("VELA_APP_NAME", "env_app");
        env::set_var("VELA_APP_DEBUG", "true");
        env::set_var("VELA_NEW_FEATURE", "cache");

        let mut loader = ConfigLoader::new()
            .add_validator("app.timeout".to_string(), RangeValidator { min: Some(1), max: Some(300) });

        loader.load().unwrap();

        // Verify hierarchy
        assert_eq!(loader.get_string("app.name"), Some("env_app".to_string())); // env overrides file
        assert_eq!(loader.get_bool("app.debug").unwrap().unwrap(), true); // only in env
        assert_eq!(loader.get_int("app.timeout").unwrap().unwrap(), 30); // from file
        assert_eq!(loader.get_string("new_feature"), Some("cache".to_string())); // only in env

        // Clean up
        env::remove_var("VELA_APP_NAME");
        env::remove_var("VELA_APP_DEBUG");
        env::remove_var("VELA_NEW_FEATURE");
        fs::remove_file("config.json").unwrap();
    }

    #[tokio::test]
    async fn test_nested_config_structures() {
        let nested_config = r#"{
            "server": {
                "host": "0.0.0.0",
                "port": 8080,
                "ssl": {
                    "enabled": true,
                    "cert": "/path/to/cert.pem"
                }
            },
            "database": {
                "primary": {
                    "host": "db1.example.com",
                    "replicas": ["db2.example.com", "db3.example.com"]
                }
            }
        }"#;

        fs::write("config.json", nested_config).unwrap();

        let loader = ConfigLoader::new();
        loader.load().unwrap();

        // Verify flattened keys
        assert_eq!(loader.get_string("server.host"), Some("0.0.0.0".to_string()));
        assert_eq!(loader.get_int("server.port").unwrap().unwrap(), 8080);
        assert_eq!(loader.get_bool("server.ssl.enabled").unwrap().unwrap(), true);
        assert_eq!(loader.get_string("server.ssl.cert"), Some("/path/to/cert.pem".to_string()));
        assert_eq!(loader.get_string("database.primary.host"), Some("db1.example.com".to_string()));
        assert_eq!(loader.get_string("database.primary.replicas[0]"), Some("db2.example.com".to_string()));

        // Clean up
        fs::remove_file("config.json").unwrap();
    }

    #[tokio::test]
    async fn test_config_reload_error_recovery() {
        let valid_config = r#"{"valid": true}"#;
        fs::write("config.json", valid_config).unwrap();

        let loader = ConfigLoader::new();
        let manager = HotReloadBuilder::new()
            .with_loader("test".to_string(), loader).unwrap()
            .build()
            .unwrap();

        // Initial load should succeed
        manager.force_reload().await.unwrap();

        // Corrupt the config file
        let invalid_config = r#"{"invalid": json"#;
        fs::write("config.json", invalid_config).unwrap();

        // Reload should fail but not crash the system
        let result = manager.force_reload().await;
        assert!(result.is_err());

        // System should still be functional
        let loader_ref = manager.get_loader("test").unwrap();
        // The old valid config should still be cached
        assert_eq!(loader_ref.lock().unwrap().get_bool("valid").unwrap().unwrap(), true);

        // Clean up
        fs::remove_file("config.json").unwrap();
    }
}