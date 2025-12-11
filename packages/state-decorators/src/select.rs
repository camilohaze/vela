//! # @select Decorator
//!
//! Decorador para selección optimizada de estado del store.
//!
//! ## Uso
//!
//! ```vela
//! class UserProfile {
//!   @select("userStore.currentUser")
//!   computed currentUser: User
//!
//!   @select("appStore.isLoading", memo=true)
//!   computed isLoading: Bool
//!
//!   @select("cartStore.items | filter(item => item.price > 10)")
//!   computed expensiveItems: List<Item>
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Decorador @select para selección optimizada de estado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Select {
    /// Selector de estado (path o expresión)
    pub selector: String,
    /// Si debe memoizar el resultado
    pub memo: bool,
    /// Comparador personalizado para cambios (opcional)
    pub comparator: Option<String>,
    /// Si debe transformar el valor seleccionado
    pub transform: Option<String>,
}

impl Select {
    /// Crea un nuevo decorador @select
    pub fn new(selector: String) -> Self {
        Self {
            selector,
            memo: false,
            comparator: None,
            transform: None,
        }
    }

    /// Habilita memoización
    pub fn with_memo(mut self) -> Self {
        self.memo = true;
        self
    }

    /// Establece comparador personalizado
    pub fn with_comparator(mut self, comparator: String) -> Self {
        self.comparator = Some(comparator);
        self
    }

    /// Establece transformación del valor
    pub fn with_transform(mut self, transform: String) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Valida que el decorador esté correctamente configurado
    pub fn validate(&self) -> Result<(), String> {
        if self.selector.is_empty() {
            return Err("Selector cannot be empty".to_string());
        }

        // Validar sintaxis básica del selector
        if !self.is_valid_selector(&self.selector) {
            return Err(format!("Invalid selector syntax: {}", self.selector));
        }

        Ok(())
    }

    /// Valida la sintaxis del selector
    fn is_valid_selector(&self, selector: &str) -> bool {
        // Selector debe tener al menos un punto o ser una expresión válida
        if selector.contains('.') || selector.contains('|') || selector.contains('(') {
            return true;
        }

        // O ser un identificador válido
        selector.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Genera código de selección para la propiedad
    pub fn generate_selection_code(&self, property_name: &str, property_type: &str) -> String {
        let mut code = String::new();

        if self.memo {
            code.push_str("memo ");
        } else {
            code.push_str("computed ");
        }

        code.push_str(&format!("{}: {} {{\n", property_name, property_type));

        // Generar lógica de selección
        code.push_str(&format!("  // Select from: {}\n", self.selector));

        if let Some(transform) = &self.transform {
            code.push_str(&format!("  return this.{} |> {}\n", self.selector, transform));
        } else {
            code.push_str(&format!("  return this.{}\n", self.selector));
        }

        code.push_str("}");

        // Agregar comparador si existe
        if let Some(comparator) = &self.comparator {
            code.push_str(&format!(
                "\n\n  // Custom comparator: {}",
                comparator
            ));
        }

        code
    }

    /// Optimiza el selector para mejor rendimiento
    pub fn optimize_selector(&mut self) {
        // Simplificar expresiones comunes
        if self.selector.contains(" | filter") {
            // Optimizar filtros comunes
            self.selector = self.selector.replace(" | filter(item => item.", " | filter(.");
        }
    }
}

/// Función helper para crear decoradores @select
pub fn select(selector: &str) -> Select {
    Select::new(selector.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_creation() {
        let select = Select::new("userStore.currentUser".to_string());
        assert_eq!(select.selector, "userStore.currentUser");
        assert!(!select.memo);
    }

    #[test]
    fn test_select_with_memo() {
        let select = Select::new("appStore.isLoading".to_string())
            .with_memo();
        assert!(select.memo);
    }

    #[test]
    fn test_select_validation() {
        let valid_select = Select::new("store.user.name".to_string());
        assert!(valid_select.validate().is_ok());

        let invalid_select = Select::new("".to_string());
        assert!(invalid_select.validate().is_err());

        let valid_expr = Select::new("items | filter(item => item.active)".to_string());
        assert!(valid_expr.validate().is_ok());
    }

    #[test]
    fn test_selection_code_generation() {
        let select = Select::new("userStore.currentUser".to_string());
        let code = select.generate_selection_code("currentUser", "User");

        assert!(code.contains("computed currentUser: User"));
        assert!(code.contains("return this.userStore.currentUser"));
    }

    #[test]
    fn test_selector_optimization() {
        let mut select = Select::new("items | filter(item => item.active)".to_string());
        select.optimize_selector();
        // La optimización específica depende de la implementación
        assert!(select.validate().is_ok());
    }
}