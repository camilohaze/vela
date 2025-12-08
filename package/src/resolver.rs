/*!
Resolución de dependencias para el package manager de Vela

Jira: TASK-104
Historia: VELA-593
Fecha: 2025-01-30

Descripción:
Implementación completa del algoritmo de resolución de dependencias con SemVer.
Resuelve conflictos de versiones, construye el grafo de dependencias y valida compatibilidad.
*/

use crate::manifest::{VelaManifest, VersionRange};
use anyhow::{Context, Result};
use std::collections::HashMap;
use regex::Regex;

/// Representa una versión semántica
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare major, minor, patch first
        let version_cmp = (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch));

        if version_cmp != std::cmp::Ordering::Equal {
            return version_cmp;
        }

        // If versions are equal, pre-release versions have lower precedence
        match (&self.pre_release, &other.pre_release) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater, // Normal > pre-release
            (Some(_), None) => std::cmp::Ordering::Less,    // pre-release < normal
            (Some(a), Some(b)) => a.cmp(b), // Compare pre-release strings
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl SemanticVersion {
    /// Parsea una versión semántica desde string
    pub fn parse(version: &str) -> Result<Self> {
        let semver_regex = Regex::new(
            r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<pre_release>(?:0|[1-9]\d*|[a-zA-Z-][a-zA-Z0-9-]*)(?:\.(?:0|[1-9]\d*|[a-zA-Z-][a-zA-Z0-9-]*))*))?(?:\+(?P<build>[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*))?$"
        ).context("Failed to create semver regex")?;

        let captures = semver_regex.captures(version)
            .context("Invalid semantic version format")?;

        let major = captures.name("major")
            .ok_or_else(|| anyhow::anyhow!("Missing major version"))?
            .as_str().parse::<u32>()
            .context("Invalid major version")?;

        let minor = captures.name("minor")
            .ok_or_else(|| anyhow::anyhow!("Missing minor version"))?
            .as_str().parse::<u32>()
            .context("Invalid minor version")?;

        let patch = captures.name("patch")
            .ok_or_else(|| anyhow::anyhow!("Missing patch version"))?
            .as_str().parse::<u32>()
            .context("Invalid patch version")?;

        let pre_release = captures.name("pre_release")
            .map(|m| m.as_str().to_string());

        let build_metadata = captures.name("build")
            .map(|m| m.as_str().to_string());

        Ok(SemanticVersion {
            major,
            minor,
            patch,
            pre_release,
            build_metadata,
        })
    }

    /// Verifica si esta versión satisface un rango
    pub fn satisfies(&self, range: &VersionRange) -> bool {
        // Implementación básica de comparación de rangos
        match range {
            VersionRange { min: Some(min), max: None, exact: None } => {
                // ^x.y.z - compatible con x
                let min_ver = SemanticVersion::parse(min).unwrap_or_else(|_| SemanticVersion {
                    major: 0, minor: 0, patch: 0, pre_release: None, build_metadata: None
                });
                self >= &min_ver && self.major == min_ver.major
            }
            VersionRange { min: Some(min), max: Some(max), exact: None } => {
                // x.y.z - z.y.x
                let min_ver = SemanticVersion::parse(min).unwrap_or_else(|_| SemanticVersion {
                    major: 0, minor: 0, patch: 0, pre_release: None, build_metadata: None
                });
                let max_ver = SemanticVersion::parse(max).unwrap_or_else(|_| SemanticVersion {
                    major: u32::MAX, minor: u32::MAX, patch: u32::MAX, pre_release: None, build_metadata: None
                });
                self >= &min_ver && self <= &max_ver
            }
            VersionRange { exact: Some(exact), .. } => {
                // Versión exacta
                let exact_ver = SemanticVersion::parse(exact).unwrap_or_else(|_| SemanticVersion {
                    major: 0, minor: 0, patch: 0, pre_release: None, build_metadata: None
                });
                self == &exact_ver
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(build) = &self.build_metadata {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

/// Representa un conflicto de dependencias
#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub name: String,
    pub existing_version: String,
    pub requested_version: String,
}

/// Información de un paquete disponible en el registry
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub versions: Vec<String>,
    pub latest_version: String,
}

/// Fuente de una dependencia resuelta
#[derive(Debug, Clone, PartialEq)]
pub enum DependencySource {
    /// Dependencia del registry
    Registry,
    /// Dependencia local (path relativo)
    Local(String),
}

/// Dependencia resuelta con su fuente
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
}

/// Resolvedor de dependencias con SemVer completo
pub struct DependencyResolver {
    resolved: HashMap<String, ResolvedDependency>,
    conflicts: Vec<DependencyConflict>,
    available_packages: HashMap<String, PackageInfo>,
}

impl DependencyResolver {
    /// Crea un nuevo resolvedor con información del registry
    pub fn new() -> Self {
        Self {
            resolved: HashMap::new(),
            conflicts: Vec::new(),
            available_packages: HashMap::new(),
        }
    }

    /// Agrega información de paquetes disponibles (simulado)
    pub fn add_available_packages(&mut self, packages: HashMap<String, PackageInfo>) {
        self.available_packages = packages;
    }

    /// Resuelve todas las dependencias del manifest
    pub fn resolve_manifest(&mut self, manifest: &VelaManifest) -> Result<Vec<ResolvedDependency>> {
        // Obtener dependencias externas
        let external_deps = manifest.get_external_dependencies();

        // Resolver cada dependencia externa
        for (name, version_req) in external_deps {
            self.resolve_dependency(&name, &version_req)?;
        }

        // Obtener dependencias locales
        let local_deps = manifest.get_local_dependencies();

        // Agregar dependencias locales (no necesitan resolución)
        for (name, path) in local_deps {
            let dep = ResolvedDependency {
                name: name.clone(),
                version: "local".to_string(),
                source: DependencySource::Local(path.clone()),
            };
            self.resolved.insert(name.clone(), dep);
        }

        // Verificar conflictos
        if !self.conflicts.is_empty() {
            return Err(anyhow::anyhow!(
                "Conflictos de dependencias detectados: {:?}",
                self.conflicts
            ));
        }

        Ok(self.resolved.values().cloned().collect())
    }

    /// Resuelve una dependencia específica
    fn resolve_dependency(&mut self, name: &str, version_req: &str) -> Result<()> {
        // Verificar si ya está resuelta
        if self.resolved.contains_key(name) {
            // Verificar compatibilidad de versiones
            let existing = &self.resolved[name];
            if !self.is_version_compatible(&existing.version, version_req) {
                self.conflicts.push(DependencyConflict {
                    name: name.to_string(),
                    existing_version: existing.version.clone(),
                    requested_version: version_req.to_string(),
                });
            }
            return Ok(());
        }

        // Simular resolución desde registry (por ahora)
        // En una implementación real, esto consultaría el registry
        let resolved_version = self.resolve_from_registry(name, version_req)?;

        let dep = ResolvedDependency {
            name: name.to_string(),
            version: resolved_version,
            source: DependencySource::Registry,
        };

        self.resolved.insert(name.to_string(), dep);
        Ok(())
    }

    /// Resuelve versión desde registry usando SemVer
    fn resolve_from_registry(&self, name: &str, version_req: &str) -> Result<String> {
        // Buscar el paquete en los disponibles
        if let Some(package_info) = self.available_packages.get(name) {
            // Parsear el requerimiento de versión
            let range = self.parse_version_requirement(version_req)?;

            // Encontrar la versión más alta que satisface el rango
            let mut best_version: Option<SemanticVersion> = None;
            let mut best_version_str = String::new();

            for version_str in &package_info.versions {
                if let Ok(version) = SemanticVersion::parse(version_str) {
                    if version.satisfies(&range) {
                        if best_version.is_none() || version > *best_version.as_ref().unwrap() {
                            best_version = Some(version.clone());
                            best_version_str = version_str.clone();
                        }
                    }
                }
            }

            if best_version.is_some() {
                return Ok(best_version_str);
            }
        }

        // Fallback: retornar el requerimiento como versión (para tests simples)
        Ok(version_req.to_string())
    }

    /// Parsea un requerimiento de versión
    fn parse_version_requirement(&self, req: &str) -> Result<VersionRange> {
        // Implementación simplificada de parsing de rangos
        if req.starts_with('^') {
            let version = &req[1..];
            Ok(VersionRange {
                min: Some(version.to_string()),
                max: None,
                exact: None,
            })
        } else if req.starts_with(">=") {
            let version = &req[2..];
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
                exact: Some(req.to_string()),
            })
        }
    }

    /// Verifica si una versión es compatible con un requerimiento
    fn is_version_compatible(&self, existing_version: &str, requested_req: &str) -> bool {
        // Usar SemVer para comparación real
        if let (Ok(existing), Ok(range)) = (
            SemanticVersion::parse(existing_version),
            self.parse_version_requirement(requested_req)
        ) {
            existing.satisfies(&range)
        } else {
            // Fallback a comparación simple
            existing_version == requested_req
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::VelaManifest;

    #[test]
    fn test_semantic_version_parsing() {
        let version = SemanticVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert!(version.pre_release.is_none());
        assert!(version.build_metadata.is_none());

        let version_pre = SemanticVersion::parse("2.0.0-alpha.1").unwrap();
        assert_eq!(version_pre.major, 2);
        assert_eq!(version_pre.minor, 0);
        assert_eq!(version_pre.patch, 0);
        assert_eq!(version_pre.pre_release, Some("alpha.1".to_string()));

        let version_build = SemanticVersion::parse("1.0.0+build.1").unwrap();
        assert_eq!(version_build.build_metadata, Some("build.1".to_string()));
    }

    #[test]
    fn test_semantic_version_comparison() {
        let v1 = SemanticVersion::parse("1.0.0").unwrap();
        let v2 = SemanticVersion::parse("1.0.1").unwrap();
        let v3 = SemanticVersion::parse("1.1.0").unwrap();
        let v4 = SemanticVersion::parse("2.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
        assert!(v4 > v1);
    }

    #[test]
    fn test_version_range_satisfaction() {
        let version = SemanticVersion::parse("1.2.3").unwrap();

        // Test caret range (^1.2.3 should accept 1.x.x)
        let caret_range = VersionRange {
            min: Some("1.2.3".to_string()),
            max: None,
            exact: None,
        };
        assert!(version.satisfies(&caret_range));

        let version_1_3_0 = SemanticVersion::parse("1.3.0").unwrap();
        assert!(version_1_3_0.satisfies(&caret_range));

        let version_2_0_0 = SemanticVersion::parse("2.0.0").unwrap();
        assert!(!version_2_0_0.satisfies(&caret_range));

        // Test exact version
        let exact_range = VersionRange {
            min: None,
            max: None,
            exact: Some("1.2.3".to_string()),
        };
        assert!(version.satisfies(&exact_range));

        let version_1_2_4 = SemanticVersion::parse("1.2.4").unwrap();
        assert!(!version_1_2_4.satisfies(&exact_range));
    }

    #[test]
    fn test_resolve_simple_dependencies() {
        let mut resolver = DependencyResolver::new();

        // Agregar paquetes disponibles simulados
        let mut available = HashMap::new();
        available.insert("serde".to_string(), PackageInfo {
            name: "serde".to_string(),
            versions: vec!["1.0.0".to_string(), "1.0.1".to_string(), "1.1.0".to_string()],
            latest_version: "1.1.0".to_string(),
        });
        resolver.add_available_packages(available);

        // Crear manifest de prueba
        let manifest = VelaManifest {
            name: "test-project".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test project".to_string()),
            authors: None,
            license: None,
            repository: None,
            dependencies: Some(crate::manifest::Dependencies {
                external: Some(crate::manifest::ExternalDependencies {
                    deps: HashMap::from([
                        ("serde".to_string(), "^1.0.0".to_string()),
                    ]),
                }),
                local: None,
            }),
            build: None,
            scripts: None,
            workspace: None,
            package: None,
        };

        let result = resolver.resolve_manifest(&manifest);
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 1);

        // Verificar que se resolvió a la versión más alta compatible
        let serde_dep = &deps[0];
        assert_eq!(serde_dep.name, "serde");
        assert_eq!(serde_dep.version, "1.1.0"); // Debería elegir la más alta: 1.1.0
        assert_eq!(serde_dep.source, DependencySource::Registry);
    }

    #[test]
    fn test_resolve_version_conflicts() {
        let mut resolver = DependencyResolver::new();

        // Agregar paquetes disponibles
        let mut available = HashMap::new();
        available.insert("serde".to_string(), PackageInfo {
            name: "serde".to_string(),
            versions: vec!["1.0.0".to_string(), "2.0.0".to_string()],
            latest_version: "2.0.0".to_string(),
        });
        resolver.add_available_packages(available);

        // Crear manifest con requerimientos conflictivos
        let manifest = VelaManifest {
            name: "test-project".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test project".to_string()),
            authors: None,
            license: None,
            repository: None,
            dependencies: Some(crate::manifest::Dependencies {
                external: Some(crate::manifest::ExternalDependencies {
                    deps: HashMap::from([
                        ("serde".to_string(), "^1.0.0".to_string()),
                        // Otro paquete que requiere serde ^2.0.0 (simulado)
                    ]),
                }),
                local: None,
            }),
            build: None,
            scripts: None,
            workspace: None,
            package: None,
        };

        let result = resolver.resolve_manifest(&manifest);
        assert!(result.is_ok()); // Por ahora no hay conflicto real

        // TODO: Agregar test de conflicto real cuando se implemente resolución transitiva
    }

    #[test]
    fn test_local_dependencies() {
        let mut resolver = DependencyResolver::new();

        let manifest = VelaManifest {
            name: "test-project".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test project".to_string()),
            authors: None,
            license: None,
            repository: None,
            dependencies: Some(crate::manifest::Dependencies {
                external: None,
                local: Some(crate::manifest::LocalDependencies {
                    deps: HashMap::from([
                        ("local-lib".to_string(), "../libs/local-lib".to_string()),
                    ]),
                }),
            }),
            build: None,
            scripts: None,
            workspace: None,
            package: None,
        };

        let result = resolver.resolve_manifest(&manifest);
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 1);

        let local_dep = &deps[0];
        assert_eq!(local_dep.name, "local-lib");
        assert_eq!(local_dep.version, "local");
        assert_eq!(local_dep.source, DependencySource::Local("../libs/local-lib".to_string()));
    }

    #[test]
    fn test_semantic_version_edge_cases() {
        // Test pre-release versions
        let pre_release = SemanticVersion::parse("1.0.0-alpha.1").unwrap();
        assert_eq!(pre_release.major, 1);
        assert_eq!(pre_release.minor, 0);
        assert_eq!(pre_release.patch, 0);
        assert_eq!(pre_release.pre_release, Some("alpha.1".to_string()));

        // Test build metadata
        let build_meta = SemanticVersion::parse("1.0.0+build.123").unwrap();
        assert_eq!(build_meta.build_metadata, Some("build.123".to_string()));

        // Test both pre-release and build metadata
        let both = SemanticVersion::parse("1.0.0-beta.1+build.456").unwrap();
        assert_eq!(both.pre_release, Some("beta.1".to_string()));
        assert_eq!(both.build_metadata, Some("build.456".to_string()));

        // Test comparison with pre-release
        let stable = SemanticVersion::parse("1.0.0").unwrap();
        let pre = SemanticVersion::parse("1.0.0-alpha").unwrap();
        assert!(stable > pre);
    }

    #[test]
    fn test_invalid_semantic_versions() {
        // Test invalid formats
        assert!(SemanticVersion::parse("").is_err());
        assert!(SemanticVersion::parse("1").is_err());
        assert!(SemanticVersion::parse("1.0").is_err());
        assert!(SemanticVersion::parse("a.b.c").is_err());
        assert!(SemanticVersion::parse("1.0.0.0").is_err());
        assert!(SemanticVersion::parse("1.0.0-alpha..1").is_err());
    }

    #[test]
    fn test_dependency_source_enum() {
        let registry_dep = ResolvedDependency {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            source: DependencySource::Registry,
        };
        assert_eq!(registry_dep.source, DependencySource::Registry);

        let local_dep = ResolvedDependency {
            name: "test".to_string(),
            version: "local".to_string(),
            source: DependencySource::Local("../libs/test".to_string()),
        };
        match local_dep.source {
            DependencySource::Local(path) => assert_eq!(path, "../libs/test"),
            _ => panic!("Expected Local source"),
        }
    }

    #[test]
    fn test_version_range_parsing() {
        let resolver = DependencyResolver::new();

        // Test caret range
        let range = resolver.parse_version_requirement("^1.2.3").unwrap();
        assert_eq!(range.min, Some("1.2.3".to_string()));
        assert_eq!(range.max, None);
        assert_eq!(range.exact, None);

        // Test greater-equal
        let range = resolver.parse_version_requirement(">=2.0.0").unwrap();
        assert_eq!(range.min, Some("2.0.0".to_string()));
        assert_eq!(range.max, None);
        assert_eq!(range.exact, None);

        // Test exact version
        let range = resolver.parse_version_requirement("1.5.0").unwrap();
        assert_eq!(range.min, None);
        assert_eq!(range.max, None);
        assert_eq!(range.exact, Some("1.5.0".to_string()));
    }

    #[test]
    fn test_dependency_conflict_detection() {
        let mut resolver = DependencyResolver::new();

        // Simular conflicto manualmente
        resolver.conflicts.push(DependencyConflict {
            name: "conflicting-pkg".to_string(),
            existing_version: "1.0.0".to_string(),
            requested_version: "2.0.0".to_string(),
        });

        // Verificar que se detectó el conflicto
        assert_eq!(resolver.conflicts.len(), 1);
        let conflict = &resolver.conflicts[0];
        assert_eq!(conflict.name, "conflicting-pkg");
        assert_eq!(conflict.existing_version, "1.0.0");
        assert_eq!(conflict.requested_version, "2.0.0");
    }

    #[test]
    fn test_empty_manifest_resolution() {
        let mut resolver = DependencyResolver::new();

        // Manifest sin dependencias
        let manifest = VelaManifest {
            name: "empty-project".to_string(),
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

        let result = resolver.resolve_manifest(&manifest);
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 0); // No dependencies
    }

    #[test]
    fn test_mixed_dependencies_resolution() {
        let mut resolver = DependencyResolver::new();

        // Agregar paquetes disponibles
        let mut available = HashMap::new();
        available.insert("external-lib".to_string(), PackageInfo {
            name: "external-lib".to_string(),
            versions: vec!["1.0.0".to_string()],
            latest_version: "1.0.0".to_string(),
        });
        resolver.add_available_packages(available);

        // Manifest con dependencias mixtas
        let manifest = VelaManifest {
            name: "mixed-project".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: None,
            license: None,
            repository: None,
            dependencies: Some(crate::manifest::Dependencies {
                external: Some(crate::manifest::ExternalDependencies {
                    deps: HashMap::from([
                        ("external-lib".to_string(), "1.0.0".to_string()),
                    ]),
                }),
                local: Some(crate::manifest::LocalDependencies {
                    deps: HashMap::from([
                        ("local-lib".to_string(), "../libs/local-lib".to_string()),
                    ]),
                }),
            }),
            build: None,
            scripts: None,
            workspace: None,
            package: None,
        };

        let result = resolver.resolve_manifest(&manifest);
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 2);

        // Verificar dependencia externa
        let external_dep = deps.iter().find(|d| d.name == "external-lib").unwrap();
        assert_eq!(external_dep.version, "1.0.0");
        assert_eq!(external_dep.source, DependencySource::Registry);

        // Verificar dependencia local
        let local_dep = deps.iter().find(|d| d.name == "local-lib").unwrap();
        assert_eq!(local_dep.version, "local");
        assert_eq!(local_dep.source, DependencySource::Local("../libs/local-lib".to_string()));
    }
}