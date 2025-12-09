/// Sistema de DevTools para debugging del state management Redux-style
/// Proporciona time-travel debugging, state inspection y action monitoring
use crate::{Store, Action, TimeTravelMiddleware};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Mensajes del protocolo DevTools
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum DevToolsMessage {
    /// Inicialización del store
    Init {
        instance_id: String,
        state: String,
        features: Vec<String>,
    },
    /// Acción dispatchada
    ActionDispatched {
        action_type: String,
        state_before: String,
        state_after: String,
        timestamp: u64,
        action_index: usize,
    },
    /// Comando de time-travel
    TimeTravel {
        target_index: usize,
    },
    /// Estado actualizado por time-travel
    StateUpdated {
        state: String,
        from_index: usize,
        to_index: usize,
    },
    /// Error en DevTools
    Error {
        message: String,
        code: String,
    },
    /// Ping/Pong para mantener conexión
    Ping,
    Pong,
}

/// Conector para comunicación con DevTools del navegador
pub struct DevToolsConnector {
    instance_id: String,
    message_queue: Arc<Mutex<Vec<DevToolsMessage>>>,
    connected: Arc<Mutex<bool>>,
    features: Vec<String>,
}

impl DevToolsConnector {
    /// Crear nuevo conector
    pub fn new(instance_id: &str) -> Self {
        DevToolsConnector {
            instance_id: instance_id.to_string(),
            message_queue: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
            features: vec![
                "timeTravel".to_string(),
                "actionLog".to_string(),
                "stateDiff".to_string(),
                "exportImport".to_string(),
            ],
        }
    }

    /// Conectar con DevTools (simulado para esta implementación)
    pub fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut connected = self.connected.lock().unwrap();
        *connected = true;
        println!("🔧 [DevTools] Connected to instance: {}", self.instance_id);
        Ok(())
    }

    /// Desconectar de DevTools
    pub fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut connected = self.connected.lock().unwrap();
        *connected = false;
        println!("🔧 [DevTools] Disconnected from instance: {}", self.instance_id);
        Ok(())
    }

    /// Verificar si está conectado
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    /// Enviar mensaje a DevTools
    pub fn send_message(&self, message: DevToolsMessage) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_connected() {
            return Ok(()); // Silently ignore if not connected
        }

        let message_json = serde_json::to_string(&message)?;
        println!("📤 [DevTools] Sending: {}", message_json);

        // En implementación real, enviar por WebSocket
        // Por ahora, solo loggear
        let mut queue = self.message_queue.lock().unwrap();
        queue.push(message);

        Ok(())
    }

    /// Recibir mensajes pendientes
    pub fn receive_messages(&self) -> Vec<DevToolsMessage> {
        let mut queue = self.message_queue.lock().unwrap();
        queue.drain(..).collect()
    }

    /// Procesar comando recibido de DevTools
    pub fn process_command(&self, command: DevToolsMessage) -> Result<Option<DevToolsCommand>, Box<dyn std::error::Error>> {
        match command {
            DevToolsMessage::TimeTravel { target_index } => {
                Ok(Some(DevToolsCommand::TimeTravel { target_index }))
            }
            DevToolsMessage::Ping => {
                self.send_message(DevToolsMessage::Pong)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

/// Comandos que DevTools puede enviar al store
pub enum DevToolsCommand {
    TimeTravel { target_index: usize },
}

/// Middleware que conecta el store con DevTools
pub struct DevToolsMiddleware<State> {
    connector: Arc<DevToolsConnector>,
    action_index: Arc<Mutex<usize>>,
    _phantom: std::marker::PhantomData<State>,
}

impl<State> DevToolsMiddleware<State>
where
    State: Clone + serde::Serialize + Send + Sync + 'static,
{
    pub fn new(connector: Arc<DevToolsConnector>) -> Self {
        DevToolsMiddleware {
            connector,
            action_index: Arc::new(Mutex::new(0)),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Inicializar DevTools con el estado inicial
    pub fn init(&self, initial_state: &State) -> Result<(), Box<dyn std::error::Error>> {
        let state_json = serde_json::to_string(initial_state)?;
        self.connector.send_message(DevToolsMessage::Init {
            instance_id: self.connector.instance_id.clone(),
            state: state_json,
            features: self.connector.features.clone(),
        })?;
        Ok(())
    }

    /// Procesar una acción y enviar info a DevTools
    pub fn process_action(
        &self,
        action: &dyn Action<State = State>,
        state_before: &State,
        state_after: &State,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = self.action_index.lock().unwrap();
        let current_index = *index;
        *index += 1;

        let state_before_json = serde_json::to_string(state_before)?;
        let state_after_json = serde_json::to_string(state_after)?;

        self.connector.send_message(DevToolsMessage::ActionDispatched {
            action_type: action.action_type().to_string(),
            state_before: state_before_json,
            state_after: state_after_json,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
            action_index: current_index,
        })?;

        Ok(())
    }
}

/// Store wrapper que incluye DevTools
pub struct DevToolsStore<T> {
    store: Store<T>,
    devtools: DevToolsMiddleware<T>,
    time_travel: TimeTravelMiddleware<T>,
}

impl<T> DevToolsStore<T>
where
    T: Clone + serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
    pub fn new(store: Store<T>, connector: Arc<DevToolsConnector>) -> Result<Self, Box<dyn std::error::Error>> {
        let devtools = DevToolsMiddleware::new(Arc::clone(&connector));
        let time_travel = TimeTravelMiddleware::new(100); // Mantener 100 estados

        // Inicializar DevTools con el estado actual
        let initial_state = (*store.get_state()).clone();
        connector.connect()?;
        devtools.init(&initial_state)?;

        Ok(DevToolsStore {
            store,
            devtools,
            time_travel,
        })
    }

    /// Dispatch con DevTools monitoring (placeholder - necesita reducer)
    pub fn dispatch(&self, action: &dyn Action<State = T>) -> Result<(), Box<dyn std::error::Error>> {
        let state_before = (*self.store.get_state()).clone();

        // TODO: Aplicar reducer aquí
        // Por ahora, solo enviar info a DevTools sin cambiar estado
        // self.store.dispatch(action)?;

        let state_after = (*self.store.get_state()).clone(); // Mismo estado por ahora

        // Enviar info a DevTools
        self.devtools.process_action(action, &state_before, &state_after)?;

        // Procesar comandos pendientes de DevTools
        let messages = self.devtools.connector.receive_messages();
        for message in messages {
            if let Some(command) = self.devtools.connector.process_command(message)? {
                match command {
                    DevToolsCommand::TimeTravel { target_index } => {
                        self.time_travel_to(target_index)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Time travel a un estado específico
    pub fn time_travel_to(&self, target_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        self.time_travel.jump_to_state(target_index, &self.store)?;

        // Notificar a DevTools
        let current_state = (*self.store.get_state()).clone();
        let state_json = serde_json::to_string(&current_state)?;
        self.devtools.connector.send_message(DevToolsMessage::StateUpdated {
            state: state_json,
            from_index: 0, // TODO: track actual from_index
            to_index: target_index,
        })?;

        Ok(())
    }

    /// Obtener estado actual
    pub fn get_state(&self) -> std::sync::RwLockReadGuard<T> {
        self.store.get_state()
    }

    /// Obtener historial de DevTools
    pub fn get_devtools_history(&self) -> Vec<String> {
        // En implementación real, devolver historial serializado
        vec!["Not implemented".to_string()]
    }

    /// Exportar estado para debugging
    pub fn export_state(&self) -> Result<String, Box<dyn std::error::Error>> {
        let state = self.store.get_state();
        Ok(serde_json::to_string(&*state)?)
    }

    /// Importar estado desde JSON
    pub fn import_state(&self, json: String) -> Result<(), Box<dyn std::error::Error>> {
        let new_state: T = serde_json::from_str(&json)?;
        self.store.set_state(new_state);
        Ok(())
    }
}

/// Inspector de estado para DevTools
pub struct StateInspector;

impl StateInspector {
    /// Crear diff entre dos estados
    pub fn diff_states<T: serde::Serialize>(before: &T, after: &T) -> Result<String, Box<dyn std::error::Error>> {
        let before_json = serde_json::to_string(before)?;
        let after_json = serde_json::to_string(after)?;

        // Simple diff - en implementación real usar una librería de diff
        if before_json == after_json {
            Ok("No changes".to_string())
        } else {
            Ok(format!("Changed from {} to {}", before_json.len(), after_json.len()))
        }
    }

    /// Formatear estado para display
    pub fn format_state<T: serde::Serialize>(state: &T) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(state)?)
    }
}

/// Macros para facilitar el uso de DevTools
#[macro_export]
macro_rules! with_devtools {
    ($store:expr, $instance_id:expr) => {{
        let connector = Arc::new(DevToolsConnector::new($instance_id));
        DevToolsStore::new($store, connector)
    }};
}

#[macro_export]
macro_rules! devtools_action {
    ($action:expr, $store:expr) => {{
        println!("🎬 [DevTools] Dispatching: {}", $action.action_type());
        $store.dispatch($action)
    }};
}