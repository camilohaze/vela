/*!
Resolución de dependencias para el package manager de Vela

Jira: TASK-103
Historia: VELA-593
Fecha: 2025-01-30

Descripción:
Implementación del algoritmo de resolución de dependencias.
Resuelve conflictos de versiones y construye el grafo de dependencias.
*/

use crate::manifest::{VelaManifest, VersionRange};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

/// Representa una dependencia resuelta con versión específica
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
}

/// Fuente de la dependencia
#[derive(Debug, Clone, PartialEq)]
pub enum DependencySource {
    /// Dependencia externa desde registry
    Registry,
    /// Dependencia local (ruta del filesystem)
    Local(String),
}

/// Resolvedor de dependencias
pub struct DependencyResolver {
    resolved: HashMap<String, ResolvedDependency>,
    conflicts: Vec<DependencyConflict>,
}

impl DependencyResolver {
    /// Crea un nuevo resolvedor
    pub fn new() -> Self {
        Self {
            resolved: HashMap::new(),
            conflicts: Vec::new(),
        }
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

    /// Simula resolución desde registry (placeholder)
    fn resolve_from_registry(&self, name: &str, version_req: &str) -> Result<String> {
        // Placeholder: siempre retorna la versión solicitada
        // En implementación real, consultaría registry y resolvería rangos
        Ok(version_req.to_string())
    }

    /// Verifica si una versión es compatible con un requerimiento
    fn is_version_compatible(&self, existing_version: &str, requested_req: &str) -> bool {
        // Placeholder: comparación simple
        // En implementación real, usaría semver
        existing_version == requested_req
    }
}

/// Representa un conflicto de dependencias
#[derive(Debug, Clone)]
pub struct DependencyConflict {
    pub name: String,
    pub existing_version: String,
    pub requested_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::VelaManifest;

    #[test]
    fn test_resolve_simple_dependencies() {
        let mut resolver = DependencyResolver::new();

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
                        ("serde".to_string(), "1.0".to_string()),
                        ("anyhow".to_string(), "1.0".to_string()),
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
        assert_eq!(deps.len(), 2);

        // Verificar que las dependencias estén resueltas
        let serde_dep = deps.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde_dep.version, "1.0");
        assert_eq!(serde_dep.source, DependencySource::Registry);
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
}