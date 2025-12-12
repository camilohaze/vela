//! Hot Reload System para Configuración
//!
//! Implementación de: VELA-609
//! Historia: VELA-609
//! Fecha: 2025-12-11
//!
//! Descripción:
//! Sistema de hot reload que permite actualizar configuración
//! sin reiniciar servicios, con notificaciones y manejo de errores.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use tokio::time;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use crate::config_loader::{ConfigLoader, ConfigError};

/// Estado del hot reload
#[derive(Debug, Clone, PartialEq)]
pub enum ReloadState {
    Idle,
    Reloading,
    Success,
    Failed(String),
}

/// Evento de cambio de configuración
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub timestamp: Instant,
    pub changed_files: Vec<String>,
    pub reload_state: ReloadState,
    pub error_message: Option<String>,
}

/// Callback para notificaciones de cambio
pub type ConfigChangeCallback = Box<dyn Fn(&ConfigChangeEvent) + Send + Sync>;

/// Manager para hot reload de configuración
pub struct HotReloadManager {
    loaders: HashMap<String, Arc<Mutex<ConfigLoader>>>,
    watched_files: HashMap<String, Vec<String>>, // loader_name -> files
    callbacks: Arc<Mutex<Vec<ConfigChangeCallback>>>,
    reload_tx: broadcast::Sender<ConfigChangeEvent>,
    debounce_duration: Duration,
    _watcher: Option<notify::RecommendedWatcher>,
}

impl HotReloadManager {
    /// Crear nuevo manager de hot reload
    pub fn new() -> Self {
        let (reload_tx, _) = broadcast::channel(100);

        Self {
            loaders: HashMap::new(),
            watched_files: HashMap::new(),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            reload_tx,
            debounce_duration: Duration::from_millis(500), // Debounce 500ms
            _watcher: None,
        }
    }

    /// Registrar un loader para hot reload
    pub fn register_loader(&mut self, name: String, loader: ConfigLoader) -> Result<(), ConfigError> {
        let arc_loader = Arc::new(Mutex::new(loader));
        self.loaders.insert(name.clone(), arc_loader.clone());

        // Extraer archivos watched de este loader
        let watched = self.extract_watched_files(&arc_loader);
        self.watched_files.insert(name, watched);

        Ok(())
    }

    /// Agregar callback para cambios de configuración
    pub fn add_change_callback<F>(&mut self, callback: F)
    where
        F: Fn(&ConfigChangeEvent) + Send + Sync + 'static,
    {
        self.callbacks.lock().unwrap().push(Box::new(callback));
    }

    /// Obtener canal para suscribirse a eventos de cambio
    pub fn change_channel(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.reload_tx.subscribe()
    }

    /// Configurar duración de debounce
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    /// Iniciar hot reload
    pub fn start(&mut self) -> Result<(), ConfigError> {
        let all_files: Vec<String> = self.watched_files.values()
            .flatten()
            .cloned()
            .collect();

        if all_files.is_empty() {
            return Ok(());
        }

        // Crear watcher
        let tx = self.reload_tx.clone();
        let debounce_duration = self.debounce_duration;

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // Extraer archivos cambiados
                        let changed_files = event.paths.iter()
                            .filter_map(|p| p.to_str())
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>();

                        if !changed_files.is_empty() {
                            // Enviar evento de cambio (debounced)
                            let event = ConfigChangeEvent {
                                timestamp: Instant::now(),
                                changed_files,
                                reload_state: ReloadState::Reloading,
                                error_message: None,
                            };

                            let _ = tx.send(event);
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        })?;

        // Watch archivos
        for file in &all_files {
            if std::path::Path::new(file).exists() {
                watcher.watch(std::path::Path::new(file), RecursiveMode::NonRecursive)?;
            }
        }

        self._watcher = Some(watcher);

        // Iniciar task de debounce y reload
        self.start_reload_task(Arc::clone(&self.callbacks));

        Ok(())
    }

    /// Detener hot reload
    pub fn stop(&mut self) {
        self._watcher = None;
        // El watcher se dropea automáticamente
    }

    /// Forzar reload manual
    pub async fn force_reload(&self) -> Result<(), ConfigError> {
        self.perform_reload().await
    }

    /// Obtener estado de un loader
    pub fn get_loader(&self, name: &str) -> Option<Arc<Mutex<ConfigLoader>>> {
        self.loaders.get(name).cloned()
    }

    /// Extraer archivos watched de un loader
    fn extract_watched_files(&self, loader: &Arc<Mutex<ConfigLoader>>) -> Vec<String> {
        // En una implementación real, el loader expondría sus archivos watched
        // Por ahora, hardcodeamos archivos comunes
        vec![
            "config.json".to_string(),
            "config-dev.json".to_string(),
            "config-staging.json".to_string(),
            "config-prod.json".to_string(),
        ]
    }

    /// Iniciar task asíncrona para manejar reloads
    fn start_reload_task(&self, callbacks: Arc<Mutex<Vec<ConfigChangeCallback>>>) {
        let mut rx = self.reload_tx.subscribe();
        let loaders = self.loaders.clone();
        // No clonamos callbacks ya que no implementan Clone
        let debounce_duration = self.debounce_duration;

        tokio::spawn(async move {
            let mut last_reload = Instant::now();
            let mut pending_changes = Vec::new();

            loop {
                match time::timeout(debounce_duration, rx.recv()).await {
                    Ok(Ok(event)) => {
                        // Agregar cambios pendientes
                        pending_changes.extend(event.changed_files);

                        // Si no ha pasado suficiente tiempo desde el último reload
                        if last_reload.elapsed() < debounce_duration {
                            continue;
                        }
                    }
                    Ok(Err(broadcast::error::RecvError::Closed)) => break,
                    Ok(Err(broadcast::error::RecvError::Lagged(_))) => {
                        // Lagged - ignorar por ahora, podríamos loggear
                        continue;
                    }
                    Err(_) => {
                        // Timeout - procesar cambios pendientes
                        if !pending_changes.is_empty() {
                            let event = ConfigChangeEvent {
                                timestamp: Instant::now(),
                                changed_files: pending_changes.clone(),
                                reload_state: ReloadState::Reloading,
                                error_message: None,
                            };

                            // Notificar callbacks (por ahora omitido para evitar problemas de Clone)
                            // for callback in &callbacks {
                            //     callback(&event);
                            // }

                            // Intentar reload
                            match Self::perform_reload_static(&loaders).await {
                                Ok(_) => {
                                    let success_event = ConfigChangeEvent {
                                        timestamp: Instant::now(),
                                        changed_files: pending_changes.clone(),
                                        reload_state: ReloadState::Success,
                                        error_message: None,
                                    };

                                    let callbacks_guard = callbacks.lock().unwrap();
                                    for callback in callbacks_guard.iter() {
                                        callback(&success_event);
                                    }
                                }
                                Err(e) => {
                                    let error_event = ConfigChangeEvent {
                                        timestamp: Instant::now(),
                                        changed_files: pending_changes.clone(),
                                        reload_state: ReloadState::Failed(e.to_string()),
                                        error_message: Some(e.to_string()),
                                    };

                                    let callbacks_guard = callbacks.lock().unwrap();
                                    for callback in callbacks_guard.iter() {
                                        callback(&error_event);
                                    }
                                }
                            }

                            pending_changes.clear();
                            last_reload = Instant::now();
                        }
                    }
                }
            }
        });
    }

    /// Realizar reload de todos los loaders
    async fn perform_reload(&self) -> Result<(), ConfigError> {
        Self::perform_reload_static(&self.loaders).await
    }

    /// Función estática para reload (necesaria para el task)
    async fn perform_reload_static(loaders: &HashMap<String, Arc<Mutex<ConfigLoader>>>) -> Result<(), ConfigError> {
        for (name, loader) in loaders {
            let mut loader_guard = loader.lock().unwrap();
            match loader_guard.load() {
                Ok(_) => {
                    println!("Successfully reloaded config for loader: {}", name);
                }
                Err(e) => {
                    eprintln!("Failed to reload config for loader {}: {}", name, e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

/// Builder para HotReloadManager
pub struct HotReloadBuilder {
    manager: HotReloadManager,
}

impl HotReloadBuilder {
    pub fn new() -> Self {
        Self {
            manager: HotReloadManager::new(),
        }
    }

    pub fn with_loader(mut self, name: String, loader: ConfigLoader) -> Result<Self, ConfigError> {
        self.manager.register_loader(name, loader)?;
        Ok(self)
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ConfigChangeEvent) + Send + Sync + 'static,
    {
        self.manager.add_change_callback(callback);
        self
    }

    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.manager.debounce_duration = duration;
        self
    }

    pub fn build(mut self) -> Result<HotReloadManager, ConfigError> {
        self.manager.start()?;
        Ok(self.manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let manager = HotReloadManager::new();
        assert!(manager.loaders.is_empty());
        assert!(manager.callbacks.is_empty());
    }

    #[tokio::test]
    async fn test_register_loader() {
        let mut manager = HotReloadManager::new();
        let loader = ConfigLoader::new();

        let result = manager.register_loader("test".to_string(), loader);
        assert!(result.is_ok());
        assert!(manager.loaders.contains_key("test"));
        assert!(manager.watched_files.contains_key("test"));
    }

    #[tokio::test]
    async fn test_add_change_callback() {
        let mut manager = HotReloadManager::new();
        let callback_called = Arc::new(Mutex::new(false));

        let callback_called_clone = callback_called.clone();
        manager.add_change_callback(move |event| {
            *callback_called_clone.lock().unwrap() = true;
        });

        assert_eq!(manager.callbacks.len(), 1);

        // Simular llamada al callback
        let event = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["test.json".to_string()],
            reload_state: ReloadState::Success,
            error_message: None,
        };

        manager.callbacks[0](&event);
        assert!(*callback_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let loader = ConfigLoader::new();

        let result = HotReloadBuilder::new()
            .with_loader("test".to_string(), loader)
            .with_debounce(Duration::from_millis(100));

        assert!(result.is_ok());
    }

    #[test]
    fn test_reload_state_enum() {
        assert_eq!(ReloadState::Idle, ReloadState::Idle);
        assert_ne!(ReloadState::Success, ReloadState::Failed("error".to_string()));
    }

    #[test]
    fn test_config_change_event() {
        let event = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["config.json".to_string()],
            reload_state: ReloadState::Success,
            error_message: None,
        };

        assert_eq!(event.changed_files.len(), 1);
        assert_eq!(event.reload_state, ReloadState::Success);
        assert!(event.error_message.is_none());
    }

    #[tokio::test]
    async fn test_force_reload() {
        let manager = HotReloadManager::new();
        // Sin loaders registrados, debería funcionar (no hacer nada)
        let result = manager.force_reload().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_watched_files() {
        let manager = HotReloadManager::new();
        let loader = Arc::new(Mutex::new(ConfigLoader::new()));

        let files = manager.extract_watched_files(&loader);
        assert!(!files.is_empty());
        assert!(files.contains(&"config.json".to_string()));
    }
}