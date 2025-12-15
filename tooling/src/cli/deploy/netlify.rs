//! Netlify deployment implementation

use super::common::{DeploymentConfig, DeploymentResult, DeploymentError, Deployer};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Netlify deployment response
#[derive(Deserialize)]
struct NetlifyDeploymentResponse {
    id: String,
    url: String,
    state: String,
}

/// Netlify site response
#[derive(Deserialize)]
struct NetlifySiteResponse {
    id: String,
    name: String,
    url: String,
}

/// Netlify deployer
pub struct NetlifyDeployer {
    client: Client,
    token: String,
}

impl NetlifyDeployer {
    /// Create a new Netlify deployer
    pub fn new() -> Result<Self, DeploymentError> {
        let token = std::env::var("NETLIFY_AUTH_TOKEN")
            .map_err(|_| DeploymentError::auth(
                "NETLIFY_AUTH_TOKEN environment variable not set. Get your token from https://app.netlify.com/user/applications#personal-access-tokens".to_string()
            ))?;

        let client = Client::new();

        Ok(Self { client, token })
    }

    /// Create or get Netlify site
    async fn ensure_site(&self, config: &DeploymentConfig) -> Result<String, DeploymentError> {
        let site_name = format!("vela-{}-{}", 
            config.project_root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("app"), 
            config.environment
        );

        // Check if site exists
        let sites_url = "https://api.netlify.com/api/v1/sites";
        let response = self.client
            .get(sites_url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| DeploymentError::platform(format!("Failed to fetch sites: {}", e)))?;

        if response.status().is_success() {
            let sites: Vec<NetlifySiteResponse> = response.json().await
                .map_err(|e| DeploymentError::platform(format!("Failed to parse sites response: {}", e)))?;

            for site in sites {
                if site.name == site_name {
                    return Ok(site.id);
                }
            }
        }

        // Create new site
        println!("   📋 Creating Netlify site: {}", site_name);

        #[derive(Serialize)]
        struct CreateSiteRequest {
            name: String,
        }

        let create_req = CreateSiteRequest {
            name: site_name,
        };

        let response = self.client
            .post(sites_url)
            .bearer_auth(&self.token)
            .json(&create_req)
            .send()
            .await
            .map_err(|e| DeploymentError::platform(format!("Failed to create site: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DeploymentError::platform(format!("Failed to create Netlify site: {}", error_text)));
        }

        let site: NetlifySiteResponse = response.json().await
            .map_err(|e| DeploymentError::platform(format!("Failed to parse site response: {}", e)))?;

        Ok(site.id)
    }

    /// Deploy to Netlify
    async fn deploy_to_netlify(&self, config: &DeploymentConfig, site_id: &str) -> Result<String, DeploymentError> {
        // Find the built files (HTML, JS, CSS, etc.)
        let build_files = self.collect_build_files(&config.build_dir)?;

        if build_files.is_empty() {
            return Err(DeploymentError::build("No build files found for Netlify deployment".to_string()));
        }

        println!("   📋 Deploying {} files to Netlify...", build_files.len());

        // Create deployment
        let deploy_url = format!("https://api.netlify.com/api/v1/sites/{}/deploys", site_id);

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
        form = form.text("title", format!("Vela deployment - {}", config.environment));

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
            return Err(DeploymentError::deployment(format!("Netlify deployment failed: {}", error_text)));
        }

        let deployment: NetlifyDeploymentResponse = response.json().await
            .map_err(|e| DeploymentError::deployment(format!("Failed to parse deployment response: {}", e)))?;

        // Wait for deployment to complete
        self.wait_for_deployment(site_id, &deployment.id).await?;

        // Get the final site URL
        let site_url = self.get_site_url(site_id).await?;

        Ok(site_url)
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
    async fn wait_for_deployment(&self, site_id: &str, deploy_id: &str) -> Result<(), DeploymentError> {
        let check_url = format!("https://api.netlify.com/api/v1/sites/{}/deploys/{}", site_id, deploy_id);

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
                let deployment: NetlifyDeploymentResponse = response.json().await
                    .map_err(|e| DeploymentError::deployment(format!("Failed to parse status response: {}", e)))?;

                match deployment.state.as_str() {
                    "ready" => return Ok(()),
                    "error" => return Err(DeploymentError::deployment("Deployment failed".to_string())),
                    _ => continue,
                }
            }
        }

        Err(DeploymentError::deployment("Deployment timeout".to_string()))
    }

    /// Get the site URL
    async fn get_site_url(&self, site_id: &str) -> Result<String, DeploymentError> {
        let site_url = format!("https://api.netlify.com/api/v1/sites/{}", site_id);

        let response = self.client
            .get(&site_url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| DeploymentError::platform(format!("Failed to get site info: {}", e)))?;

        if response.status().is_success() {
            let site: NetlifySiteResponse = response.json().await
                .map_err(|e| DeploymentError::platform(format!("Failed to parse site response: {}", e)))?;

            Ok(site.url)
        } else {
            Err(DeploymentError::platform("Failed to get site URL".to_string()))
        }
    }
}

#[async_trait::async_trait]
impl Deployer for NetlifyDeployer {
    async fn deploy(&self, config: &DeploymentConfig) -> Result<DeploymentResult, DeploymentError> {
        println!("   📋 Setting up Netlify site...");
        println!("   📋 Preparing deployment files...");
        println!("   📋 Uploading to Netlify...");

        // Ensure site exists
        let site_id = self.ensure_site(config).await?;

        // Deploy
        let site_url = self.deploy_to_netlify(config, &site_id).await?;

        let mut metadata = HashMap::new();
        metadata.insert("site_id".to_string(), site_id);

        Ok(DeploymentResult {
            success: true,
            url: Some(site_url),
            name: "vela-app".to_string(),
            version: "latest".to_string(),
            metadata,
        })
    }

    fn validate_config(&self, config: &DeploymentConfig) -> Result<(), DeploymentError> {
        // Check for Netlify token
        if std::env::var("NETLIFY_AUTH_TOKEN").is_err() {
            return Err(DeploymentError::auth(
                "NETLIFY_AUTH_TOKEN environment variable not set. Get your token from https://app.netlify.com/user/applications#personal-access-tokens".to_string()
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
            "Netlify account with Personal Access Token (NETLIFY_AUTH_TOKEN)".to_string(),
            "Netlify CLI installed (optional, for local development)".to_string(),
            "Web build output (HTML, JS, CSS) in build directory".to_string(),
        ]
    }
}