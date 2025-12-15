//! Vercel deployment implementation

use super::common::{DeploymentConfig, DeploymentResult, DeploymentError, Deployer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vercel deployment response
#[derive(Deserialize)]
struct VercelDeploymentResponse {
    id: String,
    url: String,
    state: String,
}

/// Vercel project response
#[derive(Deserialize)]
struct VercelProjectResponse {
    id: String,
    name: String,
}

/// Vercel deployer
pub struct VercelDeployer {
    client: Client,
    token: String,
}

impl VercelDeployer {
    /// Create a new Vercel deployer
    pub fn new() -> Result<Self, DeploymentError> {
        let token = std::env::var("VERCEL_TOKEN")
            .map_err(|_| DeploymentError::auth(
                "VERCEL_TOKEN environment variable not set. Get your token from https://vercel.com/account/tokens".to_string()
            ))?;

        let client = Client::new();

        Ok(Self { client, token })
    }

    /// Create or get Vercel project
    async fn ensure_project(&self, config: &DeploymentConfig) -> Result<String, DeploymentError> {
        let project_name = format!("vela-{}-{}", 
            config.project_root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("app"), 
            config.environment
        );

        // Check if project exists
        let projects_url = "https://api.vercel.com/v9/projects";
        let response = self.client
            .get(projects_url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| DeploymentError::platform(format!("Failed to fetch projects: {}", e)))?;

        if response.status().is_success() {
            let projects: serde_json::Value = response.json().await
                .map_err(|e| DeploymentError::platform(format!("Failed to parse projects response: {}", e)))?;

            if let Some(projects_array) = projects["projects"].as_array() {
                for project in projects_array {
                    if project["name"] == project_name {
                        return Ok(project["id"].as_str().unwrap().to_string());
                    }
                }
            }
        }

        // Create new project
        println!("   📋 Creating Vercel project: {}", project_name);

        #[derive(Serialize)]
        struct CreateProjectRequest {
            name: String,
            framework: Option<String>,
        }

        let create_req = CreateProjectRequest {
            name: project_name.clone(),
            framework: Some("other".to_string()), // Vela is custom framework
        };

        let response = self.client
            .post(projects_url)
            .bearer_auth(&self.token)
            .json(&create_req)
            .send()
            .await
            .map_err(|e| DeploymentError::platform(format!("Failed to create project: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DeploymentError::platform(format!("Failed to create Vercel project: {}", error_text)));
        }

        let project: VercelProjectResponse = response.json().await
            .map_err(|e| DeploymentError::platform(format!("Failed to parse project response: {}", e)))?;

        Ok(project.id)
    }

    /// Deploy to Vercel
    async fn deploy_to_vercel(&self, config: &DeploymentConfig, project_id: &str) -> Result<String, DeploymentError> {
        // Find the built files (HTML, JS, CSS, etc.)
        let build_files = self.collect_build_files(&config.build_dir)?;

        if build_files.is_empty() {
            return Err(DeploymentError::build("No build files found for Vercel deployment".to_string()));
        }

        println!("   📋 Deploying {} files to Vercel...", build_files.len());

        // Create deployment request
        let deploy_url = format!("https://api.vercel.com/v13/deployments?projectId={}", project_id);

        let mut form = reqwest::multipart::Form::new();

        // Add files
        for (file_path, file_name) in build_files {
            let file_content = std::fs::read(&file_path)
                .map_err(|e| DeploymentError::build(format!("Failed to read file {}: {}", file_path.display(), e)))?;

            let part = reqwest::multipart::Part::bytes(file_content)
                .file_name(file_name.clone());

            form = form.part(file_name, part);
        }

        // Add deployment metadata
        form = form.text("name", format!("vela-{}-{}", 
            config.project_root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("app"), 
            config.environment
        ));

        // Set production based on environment
        if config.environment == "prod" {
            form = form.text("production", "true");
        }

        let response = self.client
            .post(&deploy_url)
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| DeploymentError::deployment(format!("Failed to deploy: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DeploymentError::deployment(format!("Vercel deployment failed: {}", error_text)));
        }

        let deployment: VercelDeploymentResponse = response.json().await
            .map_err(|e| DeploymentError::deployment(format!("Failed to parse deployment response: {}", e)))?;

        // Wait for deployment to complete
        self.wait_for_deployment(&deployment.id).await?;

        Ok(deployment.url)
    }

    /// Collect build files for deployment
    fn collect_build_files(&self, build_dir: &std::path::Path) -> Result<Vec<(std::path::PathBuf, String)>, DeploymentError> {
        let mut files = Vec::new();

        fn collect_recursive(dir: &std::path::Path, base: &std::path::Path, files: &mut Vec<(std::path::PathBuf, String)>) -> Result<(), DeploymentError> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)
                    .map_err(|e| DeploymentError::build(format!("Failed to read directory: {}", e)))? {
                    let entry = entry.map_err(|e| DeploymentError::build(format!("Failed to read entry: {}", e)))?;
                    let path = entry.path();

                    if path.is_dir() {
                        collect_recursive(&path, base, files)?;
                    } else {
                        // Only include web assets
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if matches!(ext, "html" | "js" | "css" | "json" | "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "woff" | "woff2") {
                                let relative_path = path.strip_prefix(base)
                                    .map_err(|e| DeploymentError::build(format!("Failed to get relative path: {}", e)))?;
                                let file_name = relative_path.to_string_lossy().to_string();
                                files.push((path, file_name));
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        collect_recursive(build_dir, build_dir, &mut files)?;
        Ok(files)
    }

    /// Wait for deployment to complete
    async fn wait_for_deployment(&self, deployment_id: &str) -> Result<(), DeploymentError> {
        let check_url = format!("https://api.vercel.com/v13/deployments/{}", deployment_id);

        println!("   ⏳ Waiting for deployment to complete...");

        for _ in 0..30 { // Wait up to 5 minutes (30 * 10s)
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            let response = self.client
                .get(&check_url)
                .bearer_auth(&self.token)
                .send()
                .await
                .map_err(|e| DeploymentError::deployment(format!("Failed to check deployment status: {}", e)))?;

            if response.status().is_success() {
                let deployment: VercelDeploymentResponse = response.json().await
                    .map_err(|e| DeploymentError::deployment(format!("Failed to parse status response: {}", e)))?;

                match deployment.state.as_str() {
                    "READY" => return Ok(()),
                    "ERROR" => return Err(DeploymentError::deployment("Deployment failed".to_string())),
                    _ => continue,
                }
            }
        }

        Err(DeploymentError::deployment("Deployment timeout".to_string()))
    }
}

#[async_trait::async_trait]
impl Deployer for VercelDeployer {
    async fn deploy(&self, config: &DeploymentConfig) -> Result<DeploymentResult, DeploymentError> {
        println!("   📋 Setting up Vercel project...");
        println!("   📋 Preparing deployment files...");
        println!("   📋 Uploading to Vercel...");

        // Ensure project exists
        let project_id = self.ensure_project(config).await?;

        // Deploy
        let deployment_url = self.deploy_to_vercel(config, &project_id).await?;

        let mut metadata = HashMap::new();
        metadata.insert("project_id".to_string(), project_id);

        Ok(DeploymentResult {
            success: true,
            url: Some(deployment_url),
            name: "vela-app".to_string(),
            version: "latest".to_string(),
            metadata,
        })
    }

    fn validate_config(&self, config: &DeploymentConfig) -> Result<(), DeploymentError> {
        // Check for Vercel token
        if std::env::var("VERCEL_TOKEN").is_err() {
            return Err(DeploymentError::auth(
                "VERCEL_TOKEN environment variable not set. Get your token from https://vercel.com/account/tokens".to_string()
            ));
        }

        // Check if build directory exists and has web assets
        if !config.build_dir.exists() {
            return Err(DeploymentError::build(
                format!("Build directory does not exist: {}", config.build_dir.display())
            ));
        }

        Ok(())
    }

    fn get_requirements(&self) -> Vec<String> {
        vec![
            "Vercel account with API token (VERCEL_TOKEN)".to_string(),
            "Vercel CLI installed (optional, for local development)".to_string(),
            "Web build output (HTML, JS, CSS) in build directory".to_string(),
        ]
    }
}