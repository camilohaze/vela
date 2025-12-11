//! # @connect Decorator
//!
//! Decorador para conectar componentes/widgets a stores globales.
//!
//! ## Uso
//!
//! ```vela
//! @connect(AppStore)
//! component CounterWidget {
//!   // El store se inyecta automáticamente
//!   store: AppStore = inject(AppStore)
//!
//!   fn render() -> Widget {
//!     Text("Count: ${this.store.count}")
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Decorador @connect para conectar componentes a stores globales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connect {
    /// Nombre del store al que conectar
    pub store_name: String,
    /// Propiedades específicas a conectar (opcional)
    pub properties: Option<Vec<String>>,
    /// Si debe reconectar automáticamente en cambios
    pub auto_reconnect: bool,
}

impl Connect {
    /// Crea un nuevo decorador @connect
    pub fn new(store_name: String) -> Self {
        Self {
            store_name,
            properties: None,
            auto_reconnect: true,
        }
    }

    /// Especifica propiedades específicas a conectar
    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Deshabilita la reconexión automática
    pub fn without_auto_reconnect(mut self) -> Self {
        self.auto_reconnect = false;
        self
    }

    /// Valida que el decorador esté correctamente configurado
    pub fn validate(&self) -> Result<(), String> {
        if self.store_name.is_empty() {
            return Err("Store name cannot be empty".to_string());
        }

        // Validar que el store name siga convenciones de nomenclatura
        if !self.store_name.chars().next().unwrap_or(' ').is_uppercase() {
            return Err("Store name should start with uppercase letter".to_string());
        }

        Ok(())
    }

    /// Genera código de conexión para el componente
    pub fn generate_connection_code(&self, component_name: &str) -> String {
        let store_injection = format!(
            "  store: {} = inject({})",
            self.store_name, self.store_name
        );

        let mut code = String::new();
        code.push_str(&store_injection);

        if self.auto_reconnect {
            code.push_str(&format!(
                "\n\n  // Auto-reconnection setup\n  effect {{\n    // Reconnect when store changes\n    this.store = inject({})\n  }}",
                self.store_name
            ));
        }

        if let Some(props) = &self.properties {
            code.push_str(&format!(
                "\n\n  // Connected properties: {}",
                props.join(", ")
            ));
        }

        code
    }
}

/// Función helper para crear decoradores @connect
pub fn connect(store_name: &str) -> Connect {
    Connect::new(store_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_creation() {
        let connect = Connect::new("AppStore".to_string());
        assert_eq!(connect.store_name, "AppStore");
        assert!(connect.auto_reconnect);
        assert!(connect.properties.is_none());
    }

    #[test]
    fn test_connect_with_properties() {
        let connect = Connect::new("UserStore".to_string())
            .with_properties(vec!["user".to_string(), "isLoggedIn".to_string()]);

        assert_eq!(connect.store_name, "UserStore");
        assert_eq!(connect.properties.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_connect_validation() {
        let valid_connect = Connect::new("AppStore".to_string());
        assert!(valid_connect.validate().is_ok());

        let invalid_connect = Connect::new("".to_string());
        assert!(invalid_connect.validate().is_err());

        let invalid_case = Connect::new("appStore".to_string());
        assert!(invalid_case.validate().is_err());
    }

    #[test]
    fn test_connection_code_generation() {
        let connect = Connect::new("CounterStore".to_string());
        let code = connect.generate_connection_code("CounterWidget");

        assert!(code.contains("store: CounterStore = inject(CounterStore)"));
        assert!(code.contains("effect"));
    }
}