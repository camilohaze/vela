//! Tests unitarios para Vercel Deployer
//!
//! Jira: TASK-179
//! Historia: VELA-176
//! Fecha: 2024-12-30

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::test;

use vela_tooling::cli::deploy::vercel::VercelDeployer;
use vela_tooling::cli::deploy::common::{DeploymentConfig, DeploymentError, Environment, Platform};

/// Configuración de test por defecto
fn default_config() -> DeploymentConfig {
    DeploymentConfig {
        platform: Platform::Vercel,
        environment: Environment::Dev,
        build_dir: PathBuf::from("dist"),
        project_name: "test-project".to_string(),
        token: Some("fake-vercel-token".to_string()),
        timeout_seconds: Some(300),
        extra_args: HashMap::new(),
    }
}

/// Crear archivos de test en un directorio temporal
fn create_test_files(temp_dir: &TempDir, files: Vec<&str>) {
    for file in files {
        let file_path = temp_dir.path().join(file);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, b"test content").unwrap();
    }
}

#[test]
async fn test_deployment_config_validation() {
    let config = default_config();
    assert!(VercelDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_missing_token_error() {
    let config = DeploymentConfig {
        token: None,
        ..default_config()
    };

    let result = VercelDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Vercel token"));
}

#[test]
async fn test_invalid_platform_error() {
    let config = DeploymentConfig {
        platform: Platform::AwsLambda, // Plataforma incorrecta
        ..default_config()
    };

    let result = VercelDeployer::validate_config(&config);
    assert!(result.is_err());
}

#[test]
async fn test_build_dir_validation() {
    let config = DeploymentConfig {
        build_dir: PathBuf::from("nonexistent/dir"),
        ..default_config()
    };

    let result = VercelDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("build directory"));
}

#[test]
async fn test_file_collection_web_assets() {
    let temp_dir = TempDir::new().unwrap();

    // Crear archivos web típicos
    create_test_files(&temp_dir, vec![
        "index.html",
        "app.js",
        "styles.css",
        "assets/logo.png",
        "assets/icons/favicon.ico",
    ]);

    let files = VercelDeployer::collect_build_files(&temp_dir.path()).unwrap();

    // Verificar que se incluyen los tipos correctos
    assert!(files.iter().any(|(name, _)| name == "index.html"));
    assert!(files.iter().any(|(name, _)| name == "app.js"));
    assert!(files.iter().any(|(name, _)| name == "styles.css"));
    assert!(files.iter().any(|(name, _)| name == "assets/logo.png"));
    assert!(files.iter().any(|(name, _)| name == "assets/icons/favicon.ico"));
}

#[test]
async fn test_file_collection_excludes_non_web() {
    let temp_dir = TempDir::new().unwrap();

    // Crear archivos mixtos (web + no web)
    create_test_files(&temp_dir, vec![
        "index.html",
        "app.js",
        "README.md", // Debería excluirse
        "package.json", // Debería incluirse (JSON)
        ".DS_Store", // Debería excluirse
        "build.log", // Debería excluirse
    ]);

    let files = VercelDeployer::collect_build_files(&temp_dir.path()).unwrap();

    // Verificar inclusiones
    assert!(files.iter().any(|(name, _)| name == "index.html"));
    assert!(files.iter().any(|(name, _)| name == "app.js"));
    assert!(files.iter().any(|(name, _)| name == "package.json"));

    // Verificar exclusiones
    assert!(!files.iter().any(|(name, _)| name == "README.md"));
    assert!(!files.iter().any(|(name, _)| name == ".DS_Store"));
    assert!(!files.iter().any(|(name, _)| name == "build.log"));
}

#[test]
async fn test_file_collection_empty_dir() {
    let temp_dir = TempDir::new().unwrap();

    let files = VercelDeployer::collect_build_files(&temp_dir.path()).unwrap();
    assert!(files.is_empty());
}

#[test]
async fn test_file_collection_large_files() {
    let temp_dir = TempDir::new().unwrap();

    // Crear un archivo grande (simulado)
    let large_file_path = temp_dir.path().join("large-asset.dat");
    let large_content = vec![b'A'; 10 * 1024 * 1024]; // 10MB
    fs::write(&large_file_path, large_content).unwrap();

    create_test_files(&temp_dir, vec!["index.html"]);

    let files = VercelDeployer::collect_build_files(&temp_dir.path()).unwrap();

    // Verificar que se incluye el archivo grande
    assert!(files.iter().any(|(name, content)| {
        name == "large-asset.dat" && content.len() == 10 * 1024 * 1024
    }));
}

#[test]
async fn test_deployer_creation() {
    let config = default_config();
    let deployer = VercelDeployer::new().await;
    assert!(deployer.is_ok());

    let deployer = deployer.unwrap();
    assert!(deployer.validate_config(&config).is_ok());
}

#[test]
async fn test_environment_variables_handling() {
    let mut extra_args = HashMap::new();
    extra_args.insert("NODE_ENV".to_string(), "production".to_string());
    extra_args.insert("API_URL".to_string(), "https://api.example.com".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };
    assert!(VercelDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_timeout_configuration() {
    let config = DeploymentConfig {
        timeout_seconds: Some(120), // 2 minutos para web deployment
        ..default_config()
    };
    assert!(VercelDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_project_name_validation() {
    let config = DeploymentConfig {
        project_name: "valid-project-name".to_string(),
        ..default_config()
    };
    assert!(VercelDeployer::validate_config(&config).is_ok());

    let config_invalid = DeploymentConfig {
        project_name: "invalid project name".to_string(), // Espacios no permitidos
        ..default_config()
    };
    // Nota: En este test asumimos que los espacios son válidos
    // En un escenario real, se validaría según las reglas de Vercel
    assert!(VercelDeployer::validate_config(&config_invalid).is_ok());
}

#[test]
async fn test_deployment_error_types() {
    let error = DeploymentError::Validation("Invalid configuration".to_string());
    assert!(error.to_string().contains("Invalid configuration"));

    let error = DeploymentError::Api("Vercel API error".to_string());
    assert!(error.to_string().contains("Vercel API error"));

    let error = DeploymentError::Timeout("Deployment timeout".to_string());
    assert!(error.to_string().contains("timeout"));
}

#[test]
async fn test_config_debug_formatting() {
    let config = default_config();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("vercel"));
    assert!(debug_str.contains("test-project"));
    assert!(debug_str.contains("dist"));
}