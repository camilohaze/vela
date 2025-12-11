//! Implementación del decorador `@module`
//!
//! El decorador `@module` define módulos funcionales que NO son instanciables.
//! Los módulos agrupan providers, servicios y configuraciones relacionadas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuración de un módulo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub declarations: Vec<String>,  // Servicios/componentes declarados
    pub exports: Vec<String>,       // Qué se exporta del módulo
    pub providers: Vec<String>,     // Providers disponibles
    pub imports: Vec<String>,       // Módulos importados
}

/// Decorador @module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub config: ModuleConfig,
}

impl Module {
    pub fn new(name: String, config: ModuleConfig) -> Self {
        Self { name, config }
    }

    /// Valida la configuración del módulo
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Module name cannot be empty".to_string());
        }

        // Validar que exports ⊆ declarations
        for export in &self.config.exports {
            if !self.config.declarations.contains(export) {
                return Err(format!("Export '{}' not found in declarations", export));
            }
        }

        // Validar que providers ⊆ declarations
        for provider in &self.config.providers {
            if !self.config.declarations.contains(provider) {
                return Err(format!("Provider '{}' not found in declarations", provider));
            }
        }

        Ok(())
    }

    /// Obtiene los servicios exportados por este módulo
    pub fn get_exports(&self) -> &[String] {
        &self.config.exports
    }

    /// Obtiene los providers disponibles en este módulo
    pub fn get_providers(&self) -> &[String] {
        &self.config.providers
    }
}

/// Helper para crear módulos fácilmente
pub fn module(name: &str) -> ModuleBuilder {
    ModuleBuilder::new(name)
}

/// Builder para crear módulos de forma fluida
pub struct ModuleBuilder {
    name: String,
    declarations: Vec<String>,
    exports: Vec<String>,
    providers: Vec<String>,
    imports: Vec<String>,
}

impl ModuleBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            declarations: Vec::new(),
            exports: Vec::new(),
            providers: Vec::new(),
            imports: Vec::new(),
        }
    }

    pub fn declare(mut self, declaration: &str) -> Self {
        self.declarations.push(declaration.to_string());
        self
    }

    pub fn export(mut self, export: &str) -> Self {
        self.exports.push(export.to_string());
        self
    }

    pub fn provide(mut self, provider: &str) -> Self {
        self.providers.push(provider.to_string());
        self
    }

    pub fn import(mut self, import: &str) -> Self {
        self.imports.push(import.to_string());
        self
    }

    pub fn build(self) -> Module {
        let config = ModuleConfig {
            declarations: self.declarations,
            exports: self.exports,
            providers: self.providers,
            imports: self.imports,
        };

        Module::new(self.name, config)
    }
}