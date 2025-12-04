/*!
Registry client for package downloads
*/

use crate::common::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

/// Package metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
}

/// Registry client
pub struct Registry {
    url: String,
    client: Client,
}

impl Registry {
    /// Create new registry client
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: Client::new(),
        }
    }

    /// Fetch package metadata
    pub fn fetch_metadata(&self, _name: &str) -> Result<PackageMetadata> {
        // TODO: Implement HTTP request to registry
        // For now, return stub data
        Ok(PackageMetadata {
            name: "stub".to_string(),
            version: "0.1.0".parse().unwrap(),
            description: None,
            dependencies: Vec::new(),
        })
    }

    /// Download package
    pub fn download_package(&self, _name: &str, _version: &str) -> Result<Vec<u8>> {
        // TODO: Implement package download
        Ok(Vec::new())
    }

    /// Get registry URL
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new("https://registry.velalang.org")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry() {
        let registry = Registry::new("https://example.com");
        assert_eq!(registry.url(), "https://example.com");
    }

    #[test]
    fn test_default_registry() {
        let registry = Registry::default();
        assert_eq!(registry.url(), "https://registry.velalang.org");
    }

    #[test]
    fn test_fetch_metadata_stub() {
        let registry = Registry::default();
        let result = registry.fetch_metadata("test");

        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "stub");
    }

    #[test]
    fn test_download_package_stub() {
        let registry = Registry::default();
        let result = registry.download_package("test", "1.0.0");

        assert!(result.is_ok());
    }
}
