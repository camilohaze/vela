/*
JNI Bridge para Android Render Engine

Este archivo implementa el puente FFI entre el runtime de Vela (Rust)
y el código Kotlin de Android usando JNI.

Jira: TASK-157
Historia: VELA-1167
Fecha: 2025-12-15

Funciones exportadas:
- initialize_runtime: Inicializa el runtime de Vela
- render_frame: Renderiza un frame y retorna actualizaciones
- process_event: Procesa un evento desde Android
- destroy_runtime: Libera recursos del runtime
*/

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_long};
use std::ptr;
use std::sync::Mutex;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

// Re-exportar tipos del runtime principal
use crate::runtime::{VelaRuntime, VelaConfig, VelaVDOM, VelaEvent};

lazy_static! {
    static ref RUNTIMES: Mutex<std::collections::HashMap<u64, Box<VelaRuntime>>> =
        Mutex::new(std::collections::HashMap::new());
}

static mut NEXT_RUNTIME_ID: u64 = 1;

/// Genera un ID único para el runtime
fn generate_runtime_id() -> u64 {
    unsafe {
        let id = NEXT_RUNTIME_ID;
        NEXT_RUNTIME_ID += 1;
        id
    }
}

/// Convierte un puntero C a String
fn c_str_to_string(c_str: *const c_char) -> Result<String, std::str::Utf8Error> {
    if c_str.is_null() {
        return Ok(String::new());
    }
    unsafe { CStr::from_ptr(c_str).to_str().map(|s| s.to_string()) }
}

/// Convierte String a puntero C
fn string_to_c_str(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

/// Inicializa el runtime de Vela
/// Retorna un ID único del runtime (no un puntero directo por seguridad)
#[no_mangle]
pub extern "C" fn initialize_runtime(config_json: *const c_char) -> c_long {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Info)
            .with_tag("VelaAndroid"),
    );

    log::info!("Initializing Vela Android Runtime");

    match c_str_to_string(config_json) {
        Ok(config_str) => {
            match serde_json::from_str::<VelaConfig>(&config_str) {
                Ok(config) => {
                    match VelaRuntime::new(config) {
                        Ok(runtime) => {
                            let runtime_id = generate_runtime_id();
                            let mut runtimes = RUNTIMES.lock().unwrap();
                            runtimes.insert(runtime_id, Box::new(runtime));
                            log::info!("Vela runtime initialized with ID: {}", runtime_id);
                            runtime_id as c_long
                        }
                        Err(e) => {
                            log::error!("Failed to create Vela runtime: {:?}", e);
                            0
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to parse config JSON: {:?}", e);
                    0
                }
            }
        }
        Err(e) => {
            log::error!("Failed to convert config C string: {:?}", e);
            0
        }
    }
}

/// Renderiza un frame y retorna las actualizaciones del VDOM
#[no_mangle]
pub extern "C" fn render_frame(runtime_id: c_long, vdom_json: *const c_char) -> *mut c_char {
    let runtime_id = runtime_id as u64;

    let mut runtimes = match RUNTIMES.lock() {
        Ok(r) => r,
        Err(_) => {
            log::error!("Failed to lock runtimes mutex");
            return string_to_c_str("{}".to_string());
        }
    };

    let runtime = match runtimes.get_mut(&runtime_id) {
        Some(r) => r,
        None => {
            log::error!("Runtime with ID {} not found", runtime_id);
            return string_to_c_str("{}".to_string());
        }
    };

    // Parsear VDOM actual si existe
    let current_vdom = if !vdom_json.is_null() {
        match c_str_to_string(vdom_json) {
            Ok(vdom_str) => {
                match serde_json::from_str::<VDOMSnapshot>(&vdom_str) {
                    Ok(snapshot) => Some(snapshot),
                    Err(e) => {
                        log::warn!("Failed to parse VDOM: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to convert VDOM C string: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    // Renderizar frame
    match runtime.render_frame(current_vdom) {
        Ok(updates) => {
            match serde_json::to_string(&updates) {
                Ok(json) => string_to_c_str(json),
                Err(e) => {
                    log::error!("Failed to serialize render updates: {:?}", e);
                    string_to_c_str("{}".to_string())
                }
            }
        }
        Err(e) => {
            log::error!("Render frame failed: {:?}", e);
            string_to_c_str("{}".to_string())
        }
    }
}

/// Procesa un evento desde Android
#[no_mangle]
pub extern "C" fn process_event(runtime_id: c_long, event_json: *const c_char) {
    let runtime_id = runtime_id as u64;

    let mut runtimes = match RUNTIMES.lock() {
        Ok(r) => r,
        Err(_) => {
            log::error!("Failed to lock runtimes mutex");
            return;
        }
    };

    let runtime = match runtimes.get_mut(&runtime_id) {
        Some(r) => r,
        None => {
            log::error!("Runtime with ID {} not found", runtime_id);
            return;
        }
    };

    // Parsear evento
    let event = match c_str_to_string(event_json) {
        Ok(event_str) => {
            match serde_json::from_str::<AndroidEvent>(&event_str) {
                Ok(evt) => evt,
                Err(e) => {
                    log::error!("Failed to parse event JSON: {:?}", e);
                    return;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to convert event C string: {:?}", e);
            return;
        }
    };

    // Convertir a evento Vela y procesar
    let vela_event = event.to_vela_event();
    if let Err(e) = runtime.process_event(vela_event) {
        log::error!("Failed to process event: {:?}", e);
    }
}

/// Destruye el runtime y libera recursos
#[no_mangle]
pub extern "C" fn destroy_runtime(runtime_id: c_long) {
    let runtime_id = runtime_id as u64;

    let mut runtimes = RUNTIMES.lock().unwrap();
    if runtimes.remove(&runtime_id).is_some() {
        log::info!("Runtime {} destroyed", runtime_id);
    } else {
        log::warn!("Runtime {} not found for destruction", runtime_id);
    }
}

/// Snapshot del VDOM para comunicación JNI
#[derive(Serialize, Deserialize)]
struct VDOMSnapshot {
    pub version: u64,
    pub nodes: Vec<VDOMNode>,
}

/// Nodo del VDOM
#[derive(Serialize, Deserialize)]
struct VDOMNode {
    pub id: String,
    pub component_type: String,
    pub props: serde_json::Value,
    pub children: Vec<VDOMNode>,
}

/// Evento nativo de Android
#[derive(Serialize, Deserialize)]
enum AndroidEvent {
    Tap { x: f32, y: f32 },
    Scroll { delta_x: f32, delta_y: f32 },
    TextInput { text: String },
    BackPressed,
    OrientationChanged { orientation: String },
}

impl AndroidEvent {
    /// Convierte evento Android a evento Vela
    fn to_vela_event(self) -> VelaEvent {
        match self {
            AndroidEvent::Tap { x, y } => VelaEvent::PointerDown { x, y },
            AndroidEvent::Scroll { delta_x, delta_y } => VelaEvent::Scroll { delta_x, delta_y },
            AndroidEvent::TextInput { text } => VelaEvent::TextInput { text },
            AndroidEvent::BackPressed => VelaEvent::KeyPress { key: "back".to_string() },
            AndroidEvent::OrientationChanged { orientation } => {
                VelaEvent::SystemEvent {
                    event_type: "orientation_changed".to_string(),
                    data: serde_json::json!({ "orientation": orientation }),
                }
            }
        }
    }
}

/// Liberar memoria de strings retornadas a JNI
/// Esta función debe ser llamada desde Kotlin después de usar el string
#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}