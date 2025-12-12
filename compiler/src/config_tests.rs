//! Tests unitarios para Config Management
//!
//! Jira: VELA-609
//! Historia: VELA-609

use std::fs;
use std::env;
use tempfile::NamedTempFile;
use crate::config_loader::{ConfigLoader, ConfigSource, ConfigError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_new() {
        let loader = ConfigLoader::new();
        assert_eq!(loader.sources.len(), 2);
        assert!(loader.profile.is_none());
        assert!(loader.cache.is_empty());
    }

    #[test]
    fn test_add_source() {
        let loader = ConfigLoader::new()
            .add_source(ConfigSource::File("custom.json".to_string()));

        assert_eq!(loader.sources.len(), 3);
        assert!(matches!(loader.sources[2], ConfigSource::File(ref path) if path == "custom.json"));
    }

    #[test]
    fn test_with_profile() {
        let loader = ConfigLoader::new()
            .with_profile("dev".to_string());

        assert_eq!(loader.profile, Some("dev".to_string()));
        // Debería tener config-dev.json como primera fuente
        assert_eq!(loader.sources[0], ConfigSource::File("config-dev.json".to_string()));
    }

    #[test]
    fn test_load_from_env() {
        // Set up test environment variables
        env::set_var("VELA_APP_NAME", "test_app");
        env::set_var("VELA_APP_PORT", "8080");
        env::set_var("IGNORED_VAR", "should_not_load");

        let loader = ConfigLoader::new();
        let result = loader.load_from_env().unwrap();

        assert_eq!(result.get("vela_app_name"), Some(&"test_app".to_string()));
        assert_eq!(result.get("vela_app_port"), Some(&"8080".to_string()));
        assert!(!result.contains_key("ignored_var"));

        // Clean up
        env::remove_var("VELA_APP_NAME");
        env::remove_var("VELA_APP_PORT");
        env::remove_var("IGNORED_VAR");
    }

    #[test]
    fn test_load_from_file_json() {
        let json_content = r#"{
            "app": {
                "name": "test_app",
                "port": 8080,
                "features": ["auth", "logging"]
            },
            "database": {
                "host": "localhost",
                "port": 5432
            }
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, json_content).unwrap();

        let loader = ConfigLoader::new();
        let result = loader.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(result.get("app.name"), Some(&"test_app".to_string()));
        assert_eq!(result.get("app.port"), Some(&"8080".to_string()));
        assert_eq!(result.get("database.host"), Some(&"localhost".to_string()));
        assert_eq!(result.get("database.port"), Some(&"5432".to_string()));
        assert_eq!(result.get("app.features[0]"), Some(&"auth".to_string()));
        assert_eq!(result.get("app.features[1]"), Some(&"logging".to_string()));
    }

    #[test]
    fn test_load_from_file_not_found() {
        let loader = ConfigLoader::new();
        let result = loader.load_from_file("nonexistent.json");

        assert!(matches!(result, Err(ConfigError::Io(_))));
    }

    #[test]
    fn test_load_from_file_invalid_json() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "invalid json {").unwrap();

        let loader = ConfigLoader::new();
        let result = loader.load_from_file(temp_file.path().to_str().unwrap());

        assert!(matches!(result, Err(ConfigError::Json(_))));
    }

    #[test]
    fn test_load_full_integration() {
        // Create temp config file
        let json_content = r#"{"app.name": "file_app", "app.version": "1.0"}"#;
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, json_content).unwrap();

        // Set env var
        env::set_var("VELA_APP_NAME", "env_app");

        let mut loader = ConfigLoader::new()
            .add_source(ConfigSource::File(temp_file.path().to_str().unwrap().to_string()));

        loader.load().unwrap();

        // Env var should override file
        assert_eq!(loader.get_string("vela_app_name"), Some("env_app".to_string()));
        assert_eq!(loader.get_string("app.version"), Some("1.0".to_string()));

        // Clean up
        env::remove_var("VELA_APP_NAME");
    }

    #[test]
    fn test_get_typed_values() {
        let mut loader = ConfigLoader::new();

        // Add test values
        loader.cache.insert("int_value".to_string(),
            crate::config_loader::ConfigValue {
                value: "42".to_string(),
                source: ConfigSource::Environment,
                profile: None,
            });

        loader.cache.insert("bool_value".to_string(),
            crate::config_loader::ConfigValue {
                value: "true".to_string(),
                source: ConfigSource::Environment,
                profile: None,
            });

        loader.cache.insert("invalid_int".to_string(),
            crate::config_loader::ConfigValue {
                value: "not_a_number".to_string(),
                source: ConfigSource::Environment,
                profile: None,
            });

        // Test valid conversions
        assert_eq!(loader.get_int("int_value").unwrap().unwrap(), 42);
        assert_eq!(loader.get_bool("bool_value").unwrap().unwrap(), true);

        // Test invalid conversions
        assert!(loader.get_int("invalid_int").unwrap().is_err());

        // Test missing keys
        assert!(loader.get_int("missing_key").is_none());
    }

    #[test]
    fn test_validators() {
        let loader = ConfigLoader::new()
            .add_validator("required_field".to_string(), RequiredValidator)
            .add_validator("port".to_string(), RangeValidator { min: Some(1024), max: Some(65535) })
            .add_validator("email".to_string(), EmailValidator);

        // Test required validator
        let mut test_loader = loader.clone();
        test_loader.cache.insert("required_field".to_string(), ConfigValue {
            value: "".to_string(),
            source: ConfigSource::Environment,
            profile: None,
        });

        // Esto debería fallar en load, pero como estamos testeando manualmente
        let validator = RequiredValidator;
        assert!(validator.validate("required_field", "").is_err());
        assert!(validator.validate("required_field", "value").is_ok());

        // Test range validator
        let range_validator = RangeValidator { min: Some(10), max: Some(20) };
        assert!(range_validator.validate("port", "15").is_ok());
        assert!(range_validator.validate("port", "5").is_err());
        assert!(range_validator.validate("port", "25").is_err());
        assert!(range_validator.validate("port", "not_a_number").is_err());

        // Test email validator
        let email_validator = EmailValidator;
        assert!(email_validator.validate("email", "user@example.com").is_ok());
        assert!(email_validator.validate("email", "invalid-email").is_err());
    }

    #[test]
    fn test_hot_reload_enabled() {
        let loader = ConfigLoader::new().enable_hot_reload();
        assert!(loader.hot_reload_enabled);
    }
}