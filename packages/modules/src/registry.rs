//! Registry global de módulos, paquetes y bibliotecas
//!
//! Este módulo mantiene un registro global de todos los módulos
//! registrados en el sistema para resolución de dependencias.

use crate::{Package, Library, Module};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry global thread-safe
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

#[derive(Debug)]
struct RegistryInner {
    packages: HashMap<String, Package>,
    libraries: HashMap<String, Library>,
    modules: HashMap<String, Module>,
}

impl ModuleRegistry {
    /// Crea un nuevo registry vacío
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner {
                packages: HashMap::new(),
                libraries: HashMap::new(),
                modules: HashMap::new(),
            })),
        }
    }

    /// Registra un paquete
    pub fn register_package(&self, package: Package) -> Result<(), String> {
        package.validate()?;

        let mut inner = self.inner.write().unwrap();
        if inner.packages.contains_key(&package.metadata.name) {
            return Err(format!("Package '{}' already registered", package.metadata.name));
        }

        inner.packages.insert(package.metadata.name.clone(), package);
        Ok(())
    }

    /// Registra una biblioteca
    pub fn register_library(&self, library: Library) -> Result<(), String> {
        library.validate()?;

        let mut inner = self.inner.write().unwrap();
        if inner.libraries.contains_key(&library.metadata.name) {
            return Err(format!("Library '{}' already registered", library.metadata.name));
        }

        inner.libraries.insert(library.metadata.name.clone(), library);
        Ok(())
    }

    /// Registra un módulo
    pub fn register_module(&self, module: Module) -> Result<(), String> {
        module.validate()?;

        let mut inner = self.inner.write().unwrap();
        if inner.modules.contains_key(&module.name) {
            return Err(format!("Module '{}' already registered", module.name));
        }

        inner.modules.insert(module.name.clone(), module);
        Ok(())
    }

    /// Obtiene un paquete por nombre
    pub fn get_package(&self, name: &str) -> Option<Package> {
        let inner = self.inner.read().unwrap();
        inner.packages.get(name).cloned()
    }

    /// Obtiene una biblioteca por nombre
    pub fn get_library(&self, name: &str) -> Option<Library> {
        let inner = self.inner.read().unwrap();
        inner.libraries.get(name).cloned()
    }

    /// Obtiene un módulo por nombre
    pub fn get_module(&self, name: &str) -> Option<Module> {
        let inner = self.inner.read().unwrap();
        inner.modules.get(name).cloned()
    }

    /// Lista todos los paquetes registrados
    pub fn list_packages(&self) -> Vec<String> {
        let inner = self.inner.read().unwrap();
        inner.packages.keys().cloned().collect()
    }

    /// Lista todas las bibliotecas registradas
    pub fn list_libraries(&self) -> Vec<String> {
        let inner = self.inner.read().unwrap();
        inner.libraries.keys().cloned().collect()
    }

    /// Lista todos los módulos registrados
    pub fn list_modules(&self) -> Vec<String> {
        let inner = self.inner.read().unwrap();
        inner.modules.keys().cloned().collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry global singleton
static mut GLOBAL_REGISTRY: Option<ModuleRegistry> = None;

/// Obtiene el registry global
pub fn global_registry() -> &'static ModuleRegistry {
    unsafe {
        if GLOBAL_REGISTRY.is_none() {
            GLOBAL_REGISTRY = Some(ModuleRegistry::new());
        }
        GLOBAL_REGISTRY.as_ref().unwrap()
    }
}