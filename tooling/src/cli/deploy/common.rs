//! Common types and traits for cloud deployment

use std::collections::HashMap;

/// Configuration for deployment
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    /// Target platform (aws-lambda, vercel, netlify, azure-functions)
    pub platform: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
    /// Release mode
    pub release: bool,
    /// Project root directory
    pub project_root: std::path::PathBuf,
    /// Build output directory
    pub build_dir: std::path::PathBuf,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

/// Result of a deployment operation
#[derive(Debug)]
pub struct DeploymentResult {
    /// Whether deployment was successful
    pub success: bool,
    /// Deployment URL (if applicable)
    pub url: Option<String>,
    /// Function name or app name
    pub name: String,
    /// Version/tag deployed
    pub version: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Trait for cloud deployment providers
#[async_trait::async_trait]
pub trait Deployer {
    /// Deploy the application to the cloud platform
    async fn deploy(&self, config: &DeploymentConfig) -> Result<DeploymentResult, DeploymentError>;

    /// Validate deployment configuration
    fn validate_config(&self, config: &DeploymentConfig) -> Result<(), DeploymentError>;

    /// Get platform-specific requirements
    fn get_requirements(&self) -> Vec<String>;
}

/// Deployment error types
#[derive(Debug, thiserror::Error)]
pub enum DeploymentError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Authentication error: {message}")]
    Auth { message: String },

    #[error("Build error: {message}")]
    Build { message: String },

    #[error("Deployment failed: {message}")]
    Deployment { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Platform-specific error: {message}")]
    Platform { message: String },
}

impl DeploymentError {
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config { message: message.into() }
    }

    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth { message: message.into() }
    }

    pub fn build<S: Into<String>>(message: S) -> Self {
        Self::Build { message: message.into() }
    }

    pub fn deployment<S: Into<String>>(message: S) -> Self {
        Self::Deployment { message: message.into() }
    }

    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network { message: message.into() }
    }

    pub fn platform<S: Into<String>>(message: S) -> Self {
        Self::Platform { message: message.into() }
    }
}