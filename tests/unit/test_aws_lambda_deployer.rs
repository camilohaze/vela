//! Tests unitarios para AWS Lambda Deployer
//!
//! Jira: TASK-179
//! Historia: VELA-176
//! Fecha: 2024-12-30

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::test;

use vela_tooling::cli::deploy::aws_lambda::AwsLambdaDeployer;
use vela_tooling::cli::deploy::common::{DeploymentConfig, DeploymentError, Environment, Platform};

/// Configuración de test por defecto
fn default_config() -> DeploymentConfig {
    DeploymentConfig {
        platform: Platform::AwsLambda,
        environment: Environment::Dev,
        build_dir: PathBuf::from("target/debug"),
        project_name: "test-project".to_string(),
        token: Some("fake-aws-token".to_string()),
        timeout_seconds: Some(300),
        extra_args: HashMap::new(),
    }
}

#[test]
async fn test_deployment_config_validation() {
    let config = default_config();
    assert!(AwsLambdaDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_missing_token_error() {
    let config = DeploymentConfig {
        token: None,
        ..default_config()
    };

    let result = AwsLambdaDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("AWS credentials"));
}

#[test]
async fn test_invalid_platform_error() {
    let config = DeploymentConfig {
        platform: Platform::Vercel, // Plataforma incorrecta para AWS deployer
        ..default_config()
    };

    let result = AwsLambdaDeployer::validate_config(&config);
    assert!(result.is_err());
}

#[test]
async fn test_build_dir_validation() {
    let config = DeploymentConfig {
        build_dir: PathBuf::from("nonexistent/dir"),
        ..default_config()
    };

    let result = AwsLambdaDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("build directory"));
}

#[test]
async fn test_project_name_validation() {
    let config = DeploymentConfig {
        project_name: "".to_string(), // Nombre vacío
        ..default_config()
    };

    let result = AwsLambdaDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("project name"));
}

#[test]
async fn test_environment_specific_config() {
    // Test configuración para diferentes entornos
    let dev_config = DeploymentConfig {
        environment: Environment::Dev,
        ..default_config()
    };
    assert!(AwsLambdaDeployer::validate_config(&dev_config).is_ok());

    let prod_config = DeploymentConfig {
        environment: Environment::Prod,
        ..default_config()
    };
    assert!(AwsLambdaDeployer::validate_config(&prod_config).is_ok());
}

#[test]
async fn test_timeout_configuration() {
    let config = DeploymentConfig {
        timeout_seconds: Some(60),
        ..default_config()
    };
    assert!(AwsLambdaDeployer::validate_config(&config).is_ok());

    let config_no_timeout = DeploymentConfig {
        timeout_seconds: None,
        ..default_config()
    };
    assert!(AwsLambdaDeployer::validate_config(&config_no_timeout).is_ok());
}

#[test]
async fn test_extra_args_handling() {
    let mut extra_args = HashMap::new();
    extra_args.insert("memory_size".to_string(), "256".to_string());
    extra_args.insert("timeout".to_string(), "30".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };
    assert!(AwsLambdaDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_deployer_creation() {
    let config = default_config();
    let deployer = AwsLambdaDeployer::new().await;
    assert!(deployer.is_ok());

    let deployer = deployer.unwrap();
    assert!(deployer.validate_config(&config).is_ok());
}

#[test]
async fn test_deployer_with_invalid_credentials() {
    // Este test simula credenciales inválidas
    // En un test real, usaríamos mocks del AWS SDK
    let config = DeploymentConfig {
        token: Some("invalid-token".to_string()),
        ..default_config()
    };

    let deployer = AwsLambdaDeployer::new().await.unwrap();
    let result = deployer.validate_config(&config);
    // En este caso, la validación local debería pasar
    // Los errores de credenciales vendrían en el deployment real
    assert!(result.is_ok());
}

#[test]
async fn test_deployment_error_formatting() {
    let error = DeploymentError::Validation("Test error".to_string());
    assert!(error.to_string().contains("Test error"));

    let error = DeploymentError::Timeout("Operation timed out".to_string());
    assert!(error.to_string().contains("timed out"));

    let error = DeploymentError::Api("API call failed".to_string());
    assert!(error.to_string().contains("API call failed"));
}

#[test]
async fn test_config_serialization() {
    let config = default_config();

    // Test que la configuración se puede serializar (útil para logging)
    let serialized = format!("{:?}", config);
    assert!(serialized.contains("test-project"));
    assert!(serialized.contains("aws-lambda"));
}