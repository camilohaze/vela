/*!
Implementación del formato de manifest vela.yaml

Jira: TASK-102
Historia: VELA-593
Fecha: 2025-01-30

Descripción:
Implementación completa del parser y estructuras para vela.yaml
según ADR-102. Incluye validación, serialización y tipos seguros.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::bail;

/// Representa un rango de versiones semánticas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    pub min: Option<String>,
    pub max: Option<String>,
    pub exact: Option<String>,
}

impl VersionRange {
    pub fn parse(range: &str) -> Result<Self, String> {
        // Implementación simplificada de parsing de rangos
        // Soporta: "^1.2.0", "~2.1.0", "1.0.0", ">=1.0.0"
        if range.starts_with('^') {
            let version = &range[1..];
            Ok(VersionRange {
                min: Some(version.to_string()),
                max: None,
                exact: None,
            })
        } else if range.starts_with('~') {
            let version = &range[1..];
            Ok(VersionRange {
                min: Some(version.to_string()),
                max: None,
                exact: None,
            })
        } else if range.starts_with(">=") {
            let version = &range[2..];
            Ok(VersionRange {
                min: Some(version.to_string()),
                max: None,
                exact: None,
            })
        } else {
            // Versión exacta
            Ok(VersionRange {
                min: None,
                max: None,
                exact: Some(range.to_string()),
            })
        }
    }
}

/// Dependencias externas (del registro)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependencies {
    #[serde(flatten)]
    pub deps: HashMap<String, String>,
}

/// Dependencias locales (rutas relativas)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDependencies {
    #[serde(flatten)]
    pub deps: HashMap<String, String>,
}

/// Sección de dependencias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependencies {
    pub external: Option<ExternalDependencies>,
    pub local: Option<LocalDependencies>,
}

/// Configuración de compilación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildTarget {
    #[serde(rename = "web-app")]
    WebApp,
    #[serde(rename = "mobile-app")]
    MobileApp,
    #[serde(rename = "desktop-app")]
    DesktopApp,
    #[serde(rename = "service")]
    Service,
    #[serde(rename = "library")]
    Library,
    #[serde(rename = "cli-tool")]
    CliTool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "android")]
    Android,
    #[serde(rename = "ios")]
    Ios,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "macos")]
    Macos,
    #[serde(rename = "linux")]
    Linux,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "aggressive")]
    Aggressive,
    #[serde(rename = "maximum")]
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: Option<BuildTarget>,
    pub platform: Option<Platform>,
    pub optimization: Option<OptimizationLevel>,
    #[serde(rename = "source-dir")]
    pub source_dir: Option<String>,
    #[serde(rename = "output-dir")]
    pub output_dir: Option<String>,
}

/// Scripts personalizados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scripts {
    #[serde(flatten)]
    pub scripts: HashMap<String, String>,
}

/// Configuración del workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub members: Option<Vec<String>>,
}

/// Configuración del package manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub registry: Option<String>,
    pub publish: Option<bool>,
    pub private: Option<bool>,
}

/// Manifest principal de Vela
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelaManifest {
    // Metadatos
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub repository: Option<String>,

    // Dependencias
    pub dependencies: Option<Dependencies>,

    // Configuración
    pub build: Option<BuildConfig>,
    pub scripts: Option<Scripts>,
    pub workspace: Option<WorkspaceConfig>,
    pub package: Option<PackageConfig>,
}

impl VelaManifest {
    /// Carga un manifest desde un archivo
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// Parsea un manifest desde string
    pub fn from_str(content: &str) -> anyhow::Result<Self> {
        let manifest: VelaManifest = serde_yaml::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Valida el manifest
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validar nombre
        if self.name.is_empty() {
            bail!("El nombre del proyecto no puede estar vacío");
        }

        // Validar versión semántica básica
        if !self.version.contains('.') {
            bail!("La versión debe seguir formato semántico (ej: 1.0.0)");
        }

        // Validar dependencias locales (deben ser rutas válidas)
        if let Some(deps) = &self.dependencies {
            if let Some(local) = &deps.local {
                for path in local.deps.values() {
                    if path.contains("..") && !path.starts_with("../") && !path.starts_with("./") {
                        bail!("Ruta local inválida: {}", path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Serializa el manifest a YAML
    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Obtiene todas las dependencias externas
    pub fn get_external_dependencies(&self) -> HashMap<String, String> {
        self.dependencies
            .as_ref()
            .and_then(|d| d.external.as_ref())
            .map(|e| e.deps.clone())
            .unwrap_or_default()
    }

    /// Obtiene todas las dependencias locales
    pub fn get_local_dependencies(&self) -> HashMap<String, String> {
        self.dependencies
            .as_ref()
            .and_then(|d| d.local.as_ref())
            .map(|l| l.deps.clone())
            .unwrap_or_default()
    }

    /// Obtiene un script por nombre
    pub fn get_script(&self, name: &str) -> Option<&String> {
        self.scripts
            .as_ref()
            .and_then(|s| s.scripts.get(name))
    }
}

/// Builder para crear manifests programáticamente
pub struct ManifestBuilder {
    manifest: VelaManifest,
}

impl ManifestBuilder {
    pub fn new(name: String, version: String) -> Self {
        Self {
            manifest: VelaManifest {
                name,
                version,
                description: None,
                authors: None,
                license: None,
                repository: None,
                dependencies: None,
                build: None,
                scripts: None,
                workspace: None,
                package: None,
            },
        }
    }

    pub fn description(mut self, desc: String) -> Self {
        self.manifest.description = Some(desc);
        self
    }

    pub fn author(mut self, author: String) -> Self {
        self.manifest.authors.get_or_insert_with(Vec::new).push(author);
        self
    }

    pub fn license(mut self, license: String) -> Self {
        self.manifest.license = Some(license);
        self
    }

    pub fn repository(mut self, repo: String) -> Self {
        self.manifest.repository = Some(repo);
        self
    }

    pub fn add_external_dependency(mut self, name: String, version: String) -> Self {
        let deps = self.manifest.dependencies.get_or_insert_with(|| Dependencies {
            external: None,
            local: None,
        });
        let external = deps.external.get_or_insert_with(|| ExternalDependencies {
            deps: HashMap::new(),
        });
        external.deps.insert(name, version);
        self
    }

    pub fn add_local_dependency(mut self, name: String, path: String) -> Self {
        let deps = self.manifest.dependencies.get_or_insert_with(|| Dependencies {
            external: None,
            local: None,
        });
        let local = deps.local.get_or_insert_with(|| LocalDependencies {
            deps: HashMap::new(),
        });
        local.deps.insert(name, path);
        self
    }

    pub fn add_script(mut self, name: String, command: String) -> Self {
        let scripts = self.manifest.scripts.get_or_insert_with(|| Scripts {
            scripts: HashMap::new(),
        });
        scripts.scripts.insert(name, command);
        self
    }

    pub fn build(self) -> VelaManifest {
        self.manifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let yaml = r#"
name: "test-project"
version: "1.0.0"
description: "A test project"
dependencies:
  external:
    http-client: "^1.2.0"
  local:
    utils: "packages/utils"
scripts:
  build: "vela build"
  test: "vela test"
"#;

        let manifest = VelaManifest::from_str(yaml).unwrap();
        assert_eq!(manifest.name, "test-project");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, Some("A test project".to_string()));

        let external_deps = manifest.get_external_dependencies();
        assert_eq!(external_deps.get("http-client"), Some(&"^1.2.0".to_string()));

        let local_deps = manifest.get_local_dependencies();
        assert_eq!(local_deps.get("utils"), Some(&"packages/utils".to_string()));

        assert_eq!(manifest.get_script("build"), Some(&"vela build".to_string()));
    }

    #[test]
    fn test_manifest_validation() {
        // Nombre vacío
        let invalid = VelaManifest {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: None,
            license: None,
            repository: None,
            dependencies: None,
            build: None,
            scripts: None,
            workspace: None,
            package: None,
        };
        assert!(invalid.validate().is_err());

        // Versión inválida
        let invalid = VelaManifest {
            name: "test".to_string(),
            version: "1".to_string(),
            description: None,
            authors: None,
            license: None,
            repository: None,
            dependencies: None,
            build: None,
            scripts: None,
            workspace: None,
            package: None,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_manifest_builder() {
        let manifest = ManifestBuilder::new("my-project".to_string(), "1.0.0".to_string())
            .description("A project".to_string())
            .author("Author <author@example.com>".to_string())
            .add_external_dependency("serde".to_string(), "^1.0".to_string())
            .add_script("build".to_string(), "cargo build".to_string())
            .build();

        assert_eq!(manifest.name, "my-project");
        assert_eq!(manifest.description, Some("A project".to_string()));
        assert_eq!(manifest.authors, Some(vec!["Author <author@example.com>".to_string()]));
        assert_eq!(manifest.get_external_dependencies().get("serde"), Some(&"^1.0".to_string()));
        assert_eq!(manifest.get_script("build"), Some(&"cargo build".to_string()));
    }

    #[test]
    fn test_version_range_parsing() {
        let range = VersionRange::parse("^1.2.0").unwrap();
        assert_eq!(range.min, Some("1.2.0".to_string()));

        let range = VersionRange::parse("1.0.0").unwrap();
        assert_eq!(range.exact, Some("1.0.0".to_string()));

        let range = VersionRange::parse(">=2.0.0").unwrap();
        assert_eq!(range.min, Some("2.0.0".to_string()));
    }
}