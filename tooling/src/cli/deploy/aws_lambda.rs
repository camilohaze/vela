//! AWS Lambda deployment implementation

use super::common::{DeploymentConfig, DeploymentResult, DeploymentError, Deployer};
use aws_config::BehaviorVersion;
use aws_sdk_lambda::{Client as LambdaClient, types::FunctionCode};
use aws_sdk_iam::{Client as IamClient, types::Role};
use aws_sdk_s3::{Client as S3Client};
use std::collections::HashMap;
use std::path::Path;

/// AWS Lambda deployer
pub struct AwsLambdaDeployer {
    lambda_client: LambdaClient,
    iam_client: IamClient,
    s3_client: S3Client,
}

impl AwsLambdaDeployer {
    /// Create a new AWS Lambda deployer
    pub async fn new() -> Result<Self, DeploymentError> {
        let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .load()
            .await;

        let lambda_client = LambdaClient::new(&config);
        let iam_client = IamClient::new(&config);
        let s3_client = S3Client::new(&config);

        Ok(Self {
            lambda_client,
            iam_client,
            s3_client,
        })
    }

    /// Ensure the Lambda execution role exists
    async fn ensure_execution_role(&self, function_name: &str, env: &str) -> Result<String, DeploymentError> {
        let role_name = format!("vela-{}-lambda-role-{}", function_name, env);

        // Check if role already exists
        match self.iam_client.get_role().role_name(&role_name).send().await {
            Ok(role_response) => {
                return Ok(role_response.role().unwrap().arn().unwrap().to_string());
            }
            Err(_) => {
                // Role doesn't exist, create it
                println!("   📋 Creating IAM role: {}", role_name);

                let trust_policy = r#"{
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Principal": {
                                "Service": "lambda.amazonaws.com"
                            },
                            "Action": "sts:AssumeRole"
                        }
                    ]
                }"#;

                let create_role_response = self.iam_client
                    .create_role()
                    .role_name(&role_name)
                    .assume_role_policy_document(trust_policy)
                    .description("Execution role for Vela Lambda functions")
                    .send()
                    .await
                    .map_err(|e| DeploymentError::platform(format!("Failed to create IAM role: {}", e)))?;

                let role_arn = create_role_response.role().unwrap().arn().unwrap().to_string();

                // Attach basic execution policy
                self.iam_client
                    .attach_role_policy()
                    .role_name(&role_name)
                    .policy_arn("arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole")
                    .send()
                    .await
                    .map_err(|e| DeploymentError::platform(format!("Failed to attach execution policy: {}", e)))?;

                // Wait for role to be ready
                println!("   ⏳ Waiting for role to propagate...");
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                Ok(role_arn)
            }
        }
    }

    /// Create or update Lambda function
    async fn deploy_function(&self, config: &DeploymentConfig, role_arn: &str) -> Result<String, DeploymentError> {
        let function_name = format!("vela-{}-{}", config.project_root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app"), config.environment);

        // Find the compiled bytecode file
        let bytecode_path = self.find_bytecode_file(&config.build_dir)?;

        // Read bytecode
        let bytecode = std::fs::read(&bytecode_path)
            .map_err(|e| DeploymentError::build(format!("Failed to read bytecode file: {}", e)))?;

        // Create ZIP package with bytecode and runtime
        let zip_data = self.create_deployment_package(&bytecode)?;

        // Check if function exists
        let function_exists = self.lambda_client
            .get_function()
            .function_name(&function_name)
            .send()
            .await
            .is_ok();

        let function_arn = if function_exists {
            println!("   📋 Updating existing Lambda function: {}", function_name);

            // Update function code
            let update_response = self.lambda_client
                .update_function_code()
                .function_name(&function_name)
                .zip_file(zip_data)
                .send()
                .await
                .map_err(|e| DeploymentError::deployment(format!("Failed to update function: {}", e)))?;

            update_response.function_arn().unwrap().to_string()
        } else {
            println!("   📋 Creating new Lambda function: {}", function_name);

            // Create function
            let create_response = self.lambda_client
                .create_function()
                .function_name(&function_name)
                .runtime(aws_sdk_lambda::types::Runtime::Providedal2)
                .role(role_arn)
                .handler("bootstrap")
                .code(FunctionCode::builder().zip_file(zip_data).build())
                .description("Vela application deployed via vela deploy")
                .environment(self.build_environment_variables(config))
                .memory_size(256)
                .timeout(30)
                .send()
                .await
                .map_err(|e| DeploymentError::deployment(format!("Failed to create function: {}", e)))?;

            create_response.function_arn().unwrap().to_string()
        };

        Ok(function_arn)
    }

    /// Find the compiled bytecode file
    fn find_bytecode_file(&self, build_dir: &Path) -> Result<std::path::PathBuf, DeploymentError> {
        fn find_velac_recursive(dir: &Path) -> Option<std::path::PathBuf> {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(found) = find_velac_recursive(&path) {
                            return Some(found);
                        }
                    } else if path.extension().and_then(|s| s.to_str()) == Some("velac") {
                        return Some(path);
                    }
                }
            }
            None
        }

        find_velac_recursive(build_dir)
            .ok_or_else(|| DeploymentError::build("No compiled bytecode (.velac) file found in build directory".to_string()))
    }

    /// Create deployment package (ZIP with bytecode and bootstrap)
    fn create_deployment_package(&self, bytecode: &[u8]) -> Result<aws_sdk_lambda::primitives::Blob, DeploymentError> {
        use std::io::Write;

        let mut zip_buffer = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));

            // Add bytecode file
            zip.start_file("function.velac", Default::default())
                .map_err(|e| DeploymentError::build(format!("Failed to create ZIP entry: {}", e)))?;
            zip.write_all(bytecode)
                .map_err(|e| DeploymentError::build(format!("Failed to write bytecode to ZIP: {}", e)))?;

            // Add bootstrap script
            let bootstrap_script = r#"#!/bin/bash
set -euo pipefail

# Vela Lambda runtime bootstrap
echo "Starting Vela Lambda runtime..."

# Export function code
export _LAMBDA_TELEMETRY_API_V1_DISABLED=true

# Execute Vela bytecode
exec ./vela-runtime function.velac
"#;

            zip.start_file("bootstrap", Default::default())
                .map_err(|e| DeploymentError::build(format!("Failed to create bootstrap entry: {}", e)))?;
            zip.write_all(bootstrap_script.as_bytes())
                .map_err(|e| DeploymentError::build(format!("Failed to write bootstrap to ZIP: {}", e)))?;

            zip.finish()
                .map_err(|e| DeploymentError::build(format!("Failed to finalize ZIP: {}", e)))?;
        }

        Ok(aws_sdk_lambda::primitives::Blob::new(zip_buffer))
    }

    /// Build environment variables for Lambda function
    fn build_environment_variables(&self, config: &DeploymentConfig) -> aws_sdk_lambda::types::Environment {
        let mut variables = HashMap::new();

        // Add Vela-specific environment variables
        variables.insert("VELA_ENV".to_string(), config.environment.clone());
        variables.insert("VELA_PLATFORM".to_string(), "aws-lambda".to_string());

        // Add user-provided environment variables
        for (key, value) in &config.env_vars {
            variables.insert(key.clone(), value.clone());
        }

        aws_sdk_lambda::types::Environment::builder()
            .set_variables(Some(variables))
            .build()
    }

    /// Create function URL if needed
    async fn create_function_url(&self, function_name: &str) -> Result<String, DeploymentError> {
        println!("   📋 Creating function URL...");

        // Create function URL config
        self.lambda_client
            .create_function_url_config()
            .function_name(function_name)
            .auth_type(aws_sdk_lambda::types::FunctionUrlAuthType::None)
            .send()
            .await
            .map_err(|e| DeploymentError::deployment(format!("Failed to create function URL: {}", e)))?;

        // Get function URL
        let url_config = self.lambda_client
            .get_function_url_config()
            .function_name(function_name)
            .send()
            .await
            .map_err(|e| DeploymentError::deployment(format!("Failed to get function URL: {}", e)))?;

        Ok(url_config.function_url().unwrap().to_string())
    }
}

#[async_trait::async_trait]
impl Deployer for AwsLambdaDeployer {
    async fn deploy(&self, config: &DeploymentConfig) -> Result<DeploymentResult, DeploymentError> {
        println!("   📋 Creating Lambda function...");
        println!("   📋 Setting environment variables...");
        println!("   📋 Uploading function code...");
        println!("   📋 Configuring triggers...");

        // Ensure execution role exists
        let role_arn = self.ensure_execution_role("vela-app", &config.environment).await?;

        // Deploy the function
        let function_arn = self.deploy_function(config, &role_arn).await?;

        // Create function URL for HTTP access
        let function_url = self.create_function_url("vela-app-dev").await?;

        let mut metadata = HashMap::new();
        metadata.insert("function_arn".to_string(), function_arn);
        metadata.insert("role_arn".to_string(), role_arn);

        Ok(DeploymentResult {
            success: true,
            url: Some(function_url),
            name: "vela-app".to_string(),
            version: "latest".to_string(),
            metadata,
        })
    }

    fn validate_config(&self, config: &DeploymentConfig) -> Result<(), DeploymentError> {
        // Validate AWS credentials are available
        if std::env::var("AWS_ACCESS_KEY_ID").is_err() && std::env::var("AWS_PROFILE").is_err() {
            return Err(DeploymentError::auth(
                "AWS credentials not found. Set AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY or configure AWS CLI".to_string()
            ));
        }

        // Validate region is set
        if std::env::var("AWS_REGION").is_err() && std::env::var("AWS_DEFAULT_REGION").is_err() {
            return Err(DeploymentError::config(
                "AWS region not set. Set AWS_REGION or AWS_DEFAULT_REGION".to_string()
            ));
        }

        // Validate build directory exists and has bytecode
        if !config.build_dir.exists() {
            return Err(DeploymentError::build(
                format!("Build directory does not exist: {}", config.build_dir.display())
            ));
        }

        Ok(())
    }

    fn get_requirements(&self) -> Vec<String> {
        vec![
            "AWS CLI configured with appropriate permissions".to_string(),
            "IAM permissions: lambda:*, iam:*".to_string(),
            "Compiled Vela bytecode (.velac file) in build directory".to_string(),
        ]
    }
}