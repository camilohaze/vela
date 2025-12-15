//! Tests unitarios para Netlify Deployer
//!
//! Jira: TASK-179
//! Historia: VELA-176
//! Fecha: 2024-12-30

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::test;

use vela_tooling::cli::deploy::netlify::NetlifyDeployer;
use vela_tooling::cli::deploy::common::{DeploymentConfig, DeploymentError, Environment, Platform};

/// Configuración de test por defecto
fn default_config() -> DeploymentConfig {
    DeploymentConfig {
        platform: Platform::Netlify,
        environment: Environment::Dev,
        build_dir: PathBuf::from("build"),
        project_name: "test-site".to_string(),
        token: Some("fake-netlify-token".to_string()),
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
    assert!(NetlifyDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_missing_token_error() {
    let config = DeploymentConfig {
        token: None,
        ..default_config()
    };

    let result = NetlifyDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Netlify token"));
}

#[test]
async fn test_invalid_platform_error() {
    let config = DeploymentConfig {
        platform: Platform::AwsLambda, // Plataforma incorrecta
        ..default_config()
    };

    let result = NetlifyDeployer::validate_config(&config);
    assert!(result.is_err());
}

#[test]
async fn test_build_dir_validation() {
    let config = DeploymentConfig {
        build_dir: PathBuf::from("nonexistent/dir"),
        ..default_config()
    };

    let result = NetlifyDeployer::validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("build directory"));
}

#[test]
async fn test_file_collection_static_assets() {
    let temp_dir = TempDir::new().unwrap();

    // Crear archivos estáticos típicos
    create_test_files(&temp_dir, vec![
        "index.html",
        "about.html",
        "css/main.css",
        "js/app.js",
        "images/logo.png",
        "fonts/roboto.woff2",
    ]);

    let files = NetlifyDeployer::collect_build_files(&temp_dir.path()).unwrap();

    // Verificar que se incluyen los tipos correctos
    assert!(files.iter().any(|(name, _)| name == "index.html"));
    assert!(files.iter().any(|(name, _)| name == "about.html"));
    assert!(files.iter().any(|(name, _)| name == "css/main.css"));
    assert!(files.iter().any(|(name, _)| name == "js/app.js"));
    assert!(files.iter().any(|(name, _)| name == "images/logo.png"));
    assert!(files.iter().any(|(name, _)| name == "fonts/roboto.woff2"));
}

#[test]
async fn test_file_collection_excludes_build_artifacts() {
    let temp_dir = TempDir::new().unwrap();

    // Crear archivos mixtos (útiles + artifacts)
    create_test_files(&temp_dir, vec![
        "index.html",
        "app.js",
        "_redirects", // Archivo de configuración útil
        ".gitignore", // Debería excluirse
        "build.log", // Debería excluirse
        "node_modules/package.json", // Debería excluirse
        "dist/main.js.map", // Debería excluirse
    ]);

    let files = NetlifyDeployer::collect_build_files(&temp_dir.path()).unwrap();

    // Verificar inclusiones
    assert!(files.iter().any(|(name, _)| name == "index.html"));
    assert!(files.iter().any(|(name, _)| name == "app.js"));
    assert!(files.iter().any(|(name, _)| name == "_redirects"));

    // Verificar exclusiones
    assert!(!files.iter().any(|(name, _)| name == ".gitignore"));
    assert!(!files.iter().any(|(name, _)| name == "build.log"));
    assert!(!files.iter().any(|(name, _)| name.contains("node_modules")));
    assert!(!files.iter().any(|(name, _)| name.contains(".map")));
}

#[test]
async fn test_file_collection_special_netlify_files() {
    let temp_dir = TempDir::new().unwrap();

    // Crear archivos especiales de Netlify
    create_test_files(&temp_dir, vec![
        "_redirects",
        "_headers",
        "netlify.toml",
        "index.html",
    ]);

    let files = NetlifyDeployer::collect_build_files(&temp_dir.path()).unwrap();

    // Verificar que se incluyen archivos de configuración de Netlify
    assert!(files.iter().any(|(name, _)| name == "_redirects"));
    assert!(files.iter().any(|(name, _)| name == "_headers"));
    assert!(files.iter().any(|(name, _)| name == "netlify.toml"));
    assert!(files.iter().any(|(name, _)| name == "index.html"));
}

#[test]
async fn test_file_collection_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let files = NetlifyDeployer::collect_build_files(&temp_dir.path()).unwrap();
    assert!(files.is_empty());
}

#[test]
async fn test_deployer_creation() {
    let config = default_config();
    let deployer = NetlifyDeployer::new().await;
    assert!(deployer.is_ok());

    let deployer = deployer.unwrap();
    assert!(deployer.validate_config(&config).is_ok());
}

#[test]
async fn test_build_hooks_configuration() {
    let mut extra_args = HashMap::new();
    extra_args.insert("build_hook".to_string(), "https://api.netlify.com/build_hooks/123".to_string());
    extra_args.insert("branch".to_string(), "main".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };
    assert!(NetlifyDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_custom_domain_configuration() {
    let mut extra_args = HashMap::new();
    extra_args.insert("custom_domain".to_string(), "myapp.com".to_string());
    extra_args.insert("ssl_cert".to_string(), "letsencrypt".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };
    assert!(NetlifyDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_timeout_configuration() {
    let config = DeploymentConfig {
        timeout_seconds: Some(180), // 3 minutos para static site
        ..default_config()
    };
    assert!(NetlifyDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_site_name_validation() {
    let config = DeploymentConfig {
        project_name: "valid-site-name-123".to_string(),
        ..default_config()
    };
    assert!(NetlifyDeployer::validate_config(&config).is_ok());

    let config_long = DeploymentConfig {
        project_name: "a".repeat(100), // Nombre muy largo
        ..default_config()
    };
    // En este test asumimos que los nombres largos son válidos
    // En un escenario real, se validaría según límites de Netlify
    assert!(NetlifyDeployer::validate_config(&config_long).is_ok());
}

#[test]
async fn test_deployment_error_types() {
    let error = DeploymentError::Validation("Invalid site configuration".to_string());
    assert!(error.to_string().contains("Invalid site configuration"));

    let error = DeploymentError::Api("Netlify API error".to_string());
    assert!(error.to_string().contains("Netlify API error"));

    let error = DeploymentError::Timeout("Site deployment timeout".to_string());
    assert!(error.to_string().contains("timeout"));
}

#[test]
async fn test_config_debug_formatting() {
    let config = default_config();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("netlify"));
    assert!(debug_str.contains("test-site"));
    assert!(debug_str.contains("build"));
}

#[test]
async fn test_prerendering_configuration() {
    let mut extra_args = HashMap::new();
    extra_args.insert("prerender".to_string(), "true".to_string());
    extra_args.insert("prerender_urls".to_string(), "/,/about,/contact".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };
    assert!(NetlifyDeployer::validate_config(&config).is_ok());
}

#[test]
async fn test_function_deployment() {
    let mut extra_args = HashMap::new();
    extra_args.insert("functions_dir".to_string(), "netlify/functions".to_string());
    extra_args.insert("functions_runtime".to_string(), "nodejs18".to_string());

    let config = DeploymentConfig {
        extra_args,
        ..default_config()
    };
    assert!(NetlifyDeployer::validate_config(&config).is_ok());
}