//! # Vela Runtime
//!
//! Runtime de ejecución modular para el lenguaje Vela.
//!
//! Este crate proporciona el núcleo de ejecución con:
//! - Concurrencia (actores, async, channels)
//! - Sistema reactivo (signals, computed, effects)
//! - DI container
//! - HTTP framework
//! - Event system

pub use concurrency as concurrency;
pub use reactive as reactive;
pub use http as http;
pub use events as events;
pub use di as di;
pub use state_management as state_management;

// Resilience patterns
pub mod resilience;

// Observability
pub mod observability;

// Modules
// pub mod store;  // Movido a packages/state-management
// pub mod action; // Movido a packages/state-management
// pub mod reducer; // Movido a packages/state-management

// Re-export common types
pub use state_management::{Store, Action, Reducer, ReducerBuilder, create_reducer, combine_reducers};

/// Resultado común para operaciones del runtime
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Errores del runtime
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Error de inicialización: {0}")]
    InitError(String),

    #[error("Error de ejecución: {0}")]
    ExecutionError(String),

    #[error("Error de configuración: {0}")]
    ConfigError(String),

    #[error("Error de red: {0}")]
    NetworkError(String),
}

/// Configuración principal del runtime
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RuntimeConfig {
    pub async_workers: usize,
    pub max_connections: usize,
    pub event_buffer_size: usize,
    pub di_container_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            async_workers: num_cpus::get(),
            max_connections: 1000,
            event_buffer_size: 10000,
            di_container_size: 1000,
        }
    }
}

/// Runtime principal de Vela
pub struct Runtime {
    config: RuntimeConfig,
    // Campos internos se agregarán en tareas siguientes
}

impl Runtime {
    /// Crear nuevo runtime con configuración
    pub async fn new(config: RuntimeConfig) -> RuntimeResult<Self> {
        Ok(Self { config })
    }

    /// Iniciar el runtime
    pub async fn start(&self) -> RuntimeResult<()> {
        tracing::info!("Vela Runtime iniciado con config: {:?}", self.config);
        Ok(())
    }

    /// Detener el runtime
    pub async fn shutdown(&self) -> RuntimeResult<()> {
        tracing::info!("Vela Runtime detenido");
        Ok(())
    }
}