//! Tests unitarios para Hot Reload System
//!
//! Jira: VELA-609
//! Historia: VELA-609

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use crate::hot_reload::{HotReloadManager, HotReloadBuilder, ConfigChangeEvent, ReloadState};
use crate::config_loader::ConfigLoader;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hot_reload_manager_creation() {
        let manager = HotReloadManager::new();
        assert!(manager.loaders.is_empty());
        assert!(manager.callbacks.is_empty());
        assert_eq!(manager.debounce_duration, Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_register_loader() {
        let mut manager = HotReloadManager::new();
        let loader = ConfigLoader::new();

        let result = manager.register_loader("test_loader".to_string(), loader);
        assert!(result.is_ok());

        assert!(manager.loaders.contains_key("test_loader"));
        assert!(manager.watched_files.contains_key("test_loader"));

        // Verificar que se extrajeron archivos watched
        let watched = manager.watched_files.get("test_loader").unwrap();
        assert!(!watched.is_empty());
        assert!(watched.contains(&"config.json".to_string()));
    }

    #[tokio::test]
    async fn test_add_change_callback() {
        let mut manager = HotReloadManager::new();
        let callback_called = Arc::new(Mutex::new(false));
        let callback_data = Arc::new(Mutex::new(None));

        let callback_called_clone = callback_called.clone();
        let callback_data_clone = callback_data.clone();

        manager.add_change_callback(move |event| {
            *callback_called_clone.lock().unwrap() = true;
            *callback_data_clone.lock().unwrap() = Some(event.clone());
        });

        assert_eq!(manager.callbacks.len(), 1);

        // Simular evento
        let test_event = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["test.json".to_string()],
            reload_state: ReloadState::Success,
            error_message: None,
        };

        // Llamar callback manualmente
        manager.callbacks[0](&test_event);

        // Verificar que se llamó
        assert!(*callback_called.lock().unwrap());

        // Verificar datos
        let received_event = callback_data.lock().unwrap().as_ref().unwrap().clone();
        assert_eq!(received_event.changed_files, test_event.changed_files);
        assert_eq!(received_event.reload_state, ReloadState::Success);
    }

    #[tokio::test]
    async fn test_change_channel() {
        let manager = HotReloadManager::new();
        let mut rx = manager.change_channel();

        // Enviar evento al canal
        let event = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["config.json".to_string()],
            reload_state: ReloadState::Reloading,
            error_message: None,
        };

        manager.reload_tx.send(event.clone()).unwrap();

        // Recibir evento
        let received = rx.recv().await.unwrap();
        assert_eq!(received.changed_files, event.changed_files);
        assert_eq!(received.reload_state, ReloadState::Reloading);
    }

    #[tokio::test]
    async fn test_with_debounce() {
        let manager = HotReloadManager::new().with_debounce(Duration::from_millis(100));
        assert_eq!(manager.debounce_duration, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_get_loader() {
        let mut manager = HotReloadManager::new();
        let loader = ConfigLoader::new();

        manager.register_loader("test".to_string(), loader).unwrap();

        let retrieved = manager.get_loader("test");
        assert!(retrieved.is_some());

        let retrieved = manager.get_loader("nonexistent");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let loader = ConfigLoader::new();

        let result = HotReloadBuilder::new()
            .with_loader("test".to_string(), loader)
            .unwrap()
            .with_callback(|event| {
                println!("Config changed: {:?}", event.reload_state);
            })
            .with_debounce(Duration::from_millis(200));

        assert!(result.is_ok());

        let manager = result.unwrap();
        assert!(manager.loaders.contains_key("test"));
        assert_eq!(manager.callbacks.len(), 1);
        assert_eq!(manager.debounce_duration, Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_force_reload_without_loaders() {
        let manager = HotReloadManager::new();
        let result = manager.force_reload().await;
        // Sin loaders, debería ser Ok (no hace nada)
        assert!(result.is_ok());
    }

    #[test]
    fn test_reload_state_display() {
        assert_eq!(format!("{:?}", ReloadState::Idle), "Idle");
        assert_eq!(format!("{:?}", ReloadState::Reloading), "Reloading");
        assert_eq!(format!("{:?}", ReloadState::Success), "Success");
        assert_eq!(format!("{:?}", ReloadState::Failed("error".to_string())), "Failed(\"error\")");
    }

    #[test]
    fn test_config_change_event_creation() {
        let event = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["config.json".to_string(), "secrets.json".to_string()],
            reload_state: ReloadState::Failed("Parse error".to_string()),
            error_message: Some("Invalid JSON".to_string()),
        };

        assert_eq!(event.changed_files.len(), 2);
        assert_eq!(event.reload_state, ReloadState::Failed("Parse error".to_string()));
        assert_eq!(event.error_message, Some("Invalid JSON".to_string()));
    }

    #[tokio::test]
    async fn test_start_without_files() {
        let mut manager = HotReloadManager::new();
        // Sin loaders registrados, debería funcionar
        let result = manager.start();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop() {
        let mut manager = HotReloadManager::new();
        // Start y stop deberían funcionar sin errores
        manager.start().unwrap();
        manager.stop();
        // No hay assertions específicas ya que stop() no retorna nada
    }

    #[test]
    fn test_extract_watched_files() {
        let manager = HotReloadManager::new();
        let loader = Arc::new(Mutex::new(ConfigLoader::new()));

        let files = manager.extract_watched_files(&loader);

        // Debería incluir archivos de configuración comunes
        assert!(files.contains(&"config.json".to_string()));
        assert!(files.contains(&"config-dev.json".to_string()));
        assert!(files.contains(&"config-staging.json".to_string()));
        assert!(files.contains(&"config-prod.json".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_loaders() {
        let mut manager = HotReloadManager::new();

        let loader1 = ConfigLoader::new();
        let loader2 = ConfigLoader::new().with_profile("prod".to_string());

        manager.register_loader("app".to_string(), loader1).unwrap();
        manager.register_loader("db".to_string(), loader2).unwrap();

        assert_eq!(manager.loaders.len(), 2);
        assert!(manager.loaders.contains_key("app"));
        assert!(manager.loaders.contains_key("db"));
    }

    #[tokio::test]
    async fn test_callback_with_multiple_events() {
        let mut manager = HotReloadManager::new();
        let event_count = Arc::new(Mutex::new(0));

        let event_count_clone = event_count.clone();
        manager.add_change_callback(move |event| {
            let mut count = event_count_clone.lock().unwrap();
            *count += 1;
        });

        // Simular múltiples eventos
        let event1 = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["config.json".to_string()],
            reload_state: ReloadState::Reloading,
            error_message: None,
        };

        let event2 = ConfigChangeEvent {
            timestamp: Instant::now(),
            changed_files: vec!["secrets.json".to_string()],
            reload_state: ReloadState::Success,
            error_message: None,
        };

        manager.callbacks[0](&event1);
        manager.callbacks[0](&event2);

        assert_eq!(*event_count.lock().unwrap(), 2);
    }
}