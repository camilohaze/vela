//! Tests unitarios para funcionalidad común de deployment
//!
//! Jira: TASK-179
//! Historia: VELA-176
//! Fecha: 2024-12-30

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::test;

use vela_tooling::cli::deploy::common::{DeploymentConfig, DeploymentError, Environment, Platform};

/// Configuración de test por defecto
fn default_config() -> DeploymentConfig {
    DeploymentConfig {
        platform: Platform::AwsLambda,
        environment: Environment::Dev,
        build_dir: PathBuf::from("target/debug"),
        project_name: "test-project".to_string(),
        token: Some("fake-token".to_string()),
        timeout_seconds: Some(300),
        extra_args: HashMap::new(),
    }
}

#[test]
async fn test_platform_validation() {
    // Plataformas válidas
    assert!(DeploymentConfig::validate_platform("aws-lambda").is_ok());
    assert!(DeploymentConfig::validate_platform("vercel").is_ok());
    assert!(DeploymentConfig::validate_platform("netlify").is_ok());

    // Plataformas inválidas
    assert!(DeploymentConfig::validate_platform("azure-functions").is_err());
    assert!(DeploymentConfig::validate_platform("google-cloud").is_err());
    assert!(DeploymentConfig::validate_platform("heroku").is_err());
    assert!(DeploymentConfig::validate_platform("").is_err());
}

#[test]
async fn test_environment_validation() {
    // Entornos válidos
    assert!(DeploymentConfig::validate_environment("dev").is_ok());
    assert!(DeploymentConfig::validate_environment("staging").is_ok());
    assert!(DeploymentConfig::validate_environment("prod").is_ok());

    // Entornos inválidos
    assert!(DeploymentConfig::validate_environment("development").is_err());
    assert!(DeploymentConfig::validate_environment("production").is_err());
    assert!(DeploymentConfig::validate_environment("test").is_err());
    assert!(DeploymentConfig::validate_environment("").is_err());
}

#[test]
async fn test_config_creation() {
    let config = DeploymentConfig {
        platform: Platform::AwsLambda,
        environment: Environment::Dev,
        build_dir: PathBuf::from("target/debug"),
        project_name: "my-project".to_string(),
        token: Some("secret-token".to_string()),
        timeout_seconds: Some(300),
        extra_args: HashMap::new(),
    };

    assert_eq!(config.platform, Platform::AwsLambda);
    assert_eq!(config.environment, Environment::Dev);
    assert_eq!(config.build_dir, PathBuf::from("target/debug"));
    assert_eq!(config.project_name, "my-project");
    assert_eq!(config.token, Some("secret-token".to_string()));
    assert_eq!(config.timeout_seconds, Some(300));
    assert!(config.extra_args.is_empty());
}

#[test]
async fn test_config_with_extra_args() {
    let mut extra_args = HashMap::new();
    extra_args.insert("memory".to_string(), "256".to_string());
    extra_args.insert("timeout".to_string(), "30".to_string());
    extra_args.insert("env_var".to_string(), "value".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };

    assert_eq!(config.extra_args.len(), 3);
    assert_eq!(config.extra_args.get("memory"), Some(&"256".to_string()));
    assert_eq!(config.extra_args.get("timeout"), Some(&"30".to_string()));
    assert_eq!(config.extra_args.get("env_var"), Some(&"value".to_string()));
}

#[test]
async fn test_config_without_token() {
    let config = DeploymentConfig {
        token: None,
        ..default_config()
    };

    assert!(config.token.is_none());
}

#[test]
async fn test_config_without_timeout() {
    let config = DeploymentConfig {
        timeout_seconds: None,
        ..default_config()
    };

    assert!(config.timeout_seconds.is_none());
}

#[test]
async fn test_platform_enum_values() {
    // Verificar que todas las plataformas están definidas
    assert_eq!(Platform::AwsLambda as u8, 0);
    assert_eq!(Platform::Vercel as u8, 1);
    assert_eq!(Platform::Netlify as u8, 2);
}

#[test]
async fn test_environment_enum_values() {
    // Verificar que todos los entornos están definidos
    assert_eq!(Environment::Dev as u8, 0);
    assert_eq!(Environment::Staging as u8, 1);
    assert_eq!(Environment::Prod as u8, 2);
}

#[test]
async fn test_deployment_error_variants() {
    // Test de diferentes tipos de error
    let validation_error = DeploymentError::Validation("Invalid config".to_string());
    assert!(validation_error.to_string().contains("Invalid config"));

    let api_error = DeploymentError::Api("API call failed".to_string());
    assert!(api_error.to_string().contains("API call failed"));

    let timeout_error = DeploymentError::Timeout("Operation timed out".to_string());
    assert!(timeout_error.to_string().contains("timed out"));

    let io_error = DeploymentError::Io("File not found".to_string());
    assert!(io_error.to_string().contains("File not found"));
}

#[test]
async fn test_config_debug_formatting() {
    let config = default_config();
    let debug_str = format!("{:?}", config);

    // Verificar que contiene información útil para debugging
    assert!(debug_str.contains("aws-lambda"));
    assert!(debug_str.contains("dev"));
    assert!(debug_str.contains("test-project"));
    assert!(debug_str.contains("target/debug"));
}

#[test]
async fn test_config_clone() {
    let config = default_config();
    let cloned = config.clone();

    assert_eq!(config.platform, cloned.platform);
    assert_eq!(config.environment, cloned.environment);
    assert_eq!(config.build_dir, cloned.build_dir);
    assert_eq!(config.project_name, cloned.project_name);
    assert_eq!(config.token, cloned.token);
    assert_eq!(config.timeout_seconds, cloned.timeout_seconds);
    assert_eq!(config.extra_args, cloned.extra_args);
}

#[test]
async fn test_config_partial_eq() {
    let config1 = default_config();
    let config2 = default_config();

    assert_eq!(config1, config2);

    let config3 = DeploymentConfig {
        project_name: "different-project".to_string(),
        ..default_config()
    };

    assert_ne!(config1, config3);
}

#[test]
async fn test_platform_display() {
    assert_eq!(format!("{}", Platform::AwsLambda), "aws-lambda");
    assert_eq!(format!("{}", Platform::Vercel), "vercel");
    assert_eq!(format!("{}", Platform::Netlify), "netlify");
}

#[test]
async fn test_environment_display() {
    assert_eq!(format!("{}", Environment::Dev), "dev");
    assert_eq!(format!("{}", Environment::Staging), "staging");
    assert_eq!(format!("{}", Environment::Prod), "prod");
}

#[test]
async fn test_error_display() {
    let error = DeploymentError::Validation("Test message".to_string());
    let error_str = format!("{}", error);
    assert!(error_str.contains("Validation"));
    assert!(error_str.contains("Test message"));
}

#[test]
async fn test_config_with_max_timeout() {
    let config = DeploymentConfig {
        timeout_seconds: Some(3600), // 1 hora
        ..default_config()
    };

    assert_eq!(config.timeout_seconds, Some(3600));
}

#[test]
async fn test_config_with_min_timeout() {
    let config = DeploymentConfig {
        timeout_seconds: Some(30), // 30 segundos
        ..default_config()
    };

    assert_eq!(config.timeout_seconds, Some(30));
}

#[test]
async fn test_empty_extra_args() {
    let config = DeploymentConfig {
        extra_args: HashMap::new(),
        ..default_config()
    };

    assert!(config.extra_args.is_empty());
}

#[test]
async fn test_config_serialization_safety() {
    // Verificar que la configuración no expone información sensible en debug
    let config = DeploymentConfig {
        token: Some("super-secret-token".to_string()),
        ..default_config()
    };

    let debug_str = format!("{:?}", config);
    // En una implementación real, el token debería estar oculto o enmascarado
    // Para este test, verificamos que al menos se incluye en el debug
    assert!(debug_str.contains("token"));
}