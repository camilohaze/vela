//! Widget Testing Framework para Vela
//!
//! Implementación de: TASK-113CG
//! Historia: VELA-1087
//! Fecha: 2025-12-30
//!
//! Framework integrado para testing de widgets con simulación avanzada de interacciones.
//! Combina el framework básico con las capacidades avanzadas de testing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "testing")]
use vela_testing::widget_testing::{TestApp, WidgetTester};
#[cfg(feature = "testing")]
use vela_testing::matchers::{Matcher, TextMatcher, VisibilityMatcher, StyleMatcher};
#[cfg(feature = "testing")]
use vela_testing::finders::{Finder, ByType, ByKey, ByText, Descendant};
#[cfg(feature = "testing")]
use vela_testing::interactions::{Interaction, TapInteraction, TextInputInteraction};

/// Re-exportar tipos del framework avanzado para conveniencia
#[cfg(feature = "testing")]
pub use vela_testing::widget_testing::{TestApp as AdvancedTestApp, WidgetTester as AdvancedWidgetTester};
#[cfg(feature = "testing")]
pub use vela_testing::matchers::{Matcher as AdvancedMatcher, TextMatcher as AdvancedTextMatcher};
#[cfg(feature = "testing")]
pub use vela_testing::finders::{Finder as AdvancedFinder, ByType as AdvancedByType};
#[cfg(feature = "testing")]
pub use vela_testing::interactions::{Interaction as AdvancedInteraction, TapInteraction as AdvancedTapInteraction};

/// Tipo stub para cuando la feature testing no está disponible
#[cfg(not(feature = "testing"))]
#[derive(Debug, Clone)]
pub struct TestApp;

/// Implementación stub para TestApp
#[cfg(not(feature = "testing"))]
impl TestApp {
    pub fn new() -> Self {
        Self
    }
}

/// Tipos de eventos de interacción con widgets (compatibilidad hacia atrás)
#[derive(Debug, Clone, PartialEq)]
pub enum WidgetEvent {
    Click,
    DoubleClick,
    Hover,
    Unhover,
    Focus,
    Blur,
    KeyPress(String),
    Input(String),
    Scroll(i32, i32),
    Drag(i32, i32),
    Custom(String, serde_json::Value),
}

/// Estado de un widget durante testing (compatibilidad hacia atrás)
#[derive(Debug, Clone)]
pub struct WidgetState {
    pub id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub children: Vec<String>,
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
}

impl WidgetState {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            properties: HashMap::new(),
            children: Vec::new(),
            visible: true,
            enabled: true,
            focused: false,
        }
    }

    pub fn set_property(&mut self, key: &str, value: serde_json::Value) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

/// Simulador de widgets para testing (compatibilidad hacia atrás)
pub struct WidgetSimulator {
    widgets: HashMap<String, WidgetState>,
    event_handlers: HashMap<String, Vec<Box<dyn Fn(&WidgetEvent) + Send + Sync>>>,
    event_log: Arc<Mutex<Vec<(String, WidgetEvent)>>>,
}

impl WidgetSimulator {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            event_handlers: HashMap::new(),
            event_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Crear un nuevo widget en el simulador
    pub fn create_widget(&mut self, id: &str) -> &mut WidgetState {
        let state = WidgetState::new(id);
        self.widgets.insert(id.to_string(), state);
        self.widgets.get_mut(id).unwrap()
    }

    /// Obtener estado de un widget
    pub fn get_widget(&self, id: &str) -> Option<&WidgetState> {
        self.widgets.get(id)
    }

    /// Simular un evento en un widget
    pub fn simulate_event(&mut self, widget_id: &str, event: WidgetEvent) -> Result<(), String> {
        if !self.widgets.contains_key(widget_id) {
            return Err(format!("Widget '{}' not found", widget_id));
        }

        // Log del evento
        self.event_log.lock().unwrap().push((widget_id.to_string(), event.clone()));

        // Ejecutar handlers registrados
        if let Some(handlers) = self.event_handlers.get(widget_id) {
            for handler in handlers {
                handler(&event);
            }
        }

        // Simular cambios de estado basados en el evento
        self.simulate_state_change(widget_id, &event);

        Ok(())
    }

    /// Registrar un handler de eventos para un widget
    pub fn register_event_handler<F>(&mut self, widget_id: &str, handler: F)
    where
        F: Fn(&WidgetEvent) + Send + Sync + 'static,
    {
        self.event_handlers
            .entry(widget_id.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    /// Obtener log de eventos
    pub fn get_event_log(&self) -> Vec<(String, WidgetEvent)> {
        self.event_log.lock().unwrap().clone()
    }

    /// Limpiar log de eventos
    pub fn clear_event_log(&mut self) {
        self.event_log.lock().unwrap().clear();
    }

    /// Simular cambios de estado basados en eventos
    fn simulate_state_change(&mut self, widget_id: &str, event: &WidgetEvent) {
        if let Some(widget) = self.widgets.get_mut(widget_id) {
            match event {
                WidgetEvent::Click => {
                    widget.set_property("clicked", serde_json::Value::Bool(true));
                }
                WidgetEvent::Focus => {
                    widget.focused = true;
                    widget.set_property("focused", serde_json::Value::Bool(true));
                }
                WidgetEvent::Blur => {
                    widget.focused = false;
                    widget.set_property("focused", serde_json::Value::Bool(false));
                }
                WidgetEvent::Input(text) => {
                    widget.set_property("value", serde_json::Value::String(text.clone()));
                }
                WidgetEvent::Hover => {
                    widget.set_property("hovered", serde_json::Value::Bool(true));
                }
                WidgetEvent::Unhover => {
                    widget.set_property("hovered", serde_json::Value::Bool(false));
                }
                _ => {}
            }
        }
    }
}

/// Framework de testing integrado para widgets
/// Combina capacidades básicas y avanzadas
pub struct WidgetTestRunner {
    simulator: WidgetSimulator,
    #[cfg(feature = "testing")]
    advanced_app: Option<TestApp>,
    #[cfg(not(feature = "testing"))]
    advanced_app: Option<()>, // Placeholder when testing feature is disabled
    assertions: Vec<Box<dyn Fn() -> Result<(), String> + Send + Sync>>,
}

impl WidgetTestRunner {
    pub fn new() -> Self {
        Self {
            simulator: WidgetSimulator::new(),
            #[cfg(feature = "testing")]
            advanced_app: None,
            #[cfg(not(feature = "testing"))]
            advanced_app: None,
            assertions: Vec::new(),
        }
    }

    /// Crear un test runner con capacidades avanzadas
    #[cfg(feature = "testing")]
    pub fn with_advanced_testing() -> Self {
        Self {
            simulator: WidgetSimulator::new(),
            advanced_app: Some(TestApp::new()),
            assertions: Vec::new(),
        }
    }

    /// Crear un test runner básico (sin capacidades avanzadas)
    #[cfg(not(feature = "testing"))]
    pub fn with_advanced_testing() -> Self {
        Self {
            simulator: WidgetSimulator::new(),
            advanced_app: None,
            assertions: Vec::new(),
        }
    }

    /// Obtener acceso al simulador básico
    pub fn simulator(&mut self) -> &mut WidgetSimulator {
        &mut self.simulator
    }

    /// Obtener acceso al framework avanzado
    #[cfg(feature = "testing")]
    pub fn advanced_app(&mut self) -> Option<&mut TestApp> {
        self.advanced_app.as_mut()
    }

    /// Obtener acceso al framework avanzado (retorna None cuando testing no está disponible)
    #[cfg(not(feature = "testing"))]
    pub fn advanced_app(&mut self) -> Option<&mut TestApp> {
        None
    }

    /// Agregar una aserción personalizada
    pub fn add_assertion<F>(&mut self, assertion: F)
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        self.assertions.push(Box::new(assertion));
    }

    /// Ejecutar todas las aserciones
    pub fn run_assertions(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for assertion in &self.assertions {
            if let Err(error) = assertion() {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Método helper: esperar que un widget tenga una propiedad específica
    pub fn expect_property(&mut self, widget_id: &str, property: &str, expected: serde_json::Value) {
        let widget_id = widget_id.to_string();
        let property = property.to_string();
        let expected = expected.clone();

        self.add_assertion(move || {
            match WidgetTestRunner::new().simulator().get_widget(&widget_id) {
                Some(widget) => {
                    match widget.get_property(&property) {
                        Some(actual) if actual == &expected => Ok(()),
                        Some(actual) => Err(format!(
                            "Widget '{}' property '{}' expected {:?}, got {:?}",
                            widget_id, property, expected, actual
                        )),
                        None => Err(format!(
                            "Widget '{}' does not have property '{}'",
                            widget_id, property
                        )),
                    }
                }
                None => Err(format!("Widget '{}' not found", widget_id)),
            }
        });
    }

    /// Usar matchers avanzados
    #[cfg(feature = "testing")]
    pub fn expect_widget(&mut self, finder: impl Into<Box<dyn Finder>>, matcher: impl Into<Box<dyn Matcher>>) {
        let finder = finder.into();
        let matcher = matcher.into();

        self.add_assertion(move || {
            if let Some(app) = &WidgetTestRunner::with_advanced_testing().advanced_app {
                let tester = WidgetTester::new(app);
                match tester.find(finder) {
                    Ok(widgets) => {
                        if widgets.is_empty() {
                            return Err("No widgets found matching criteria".to_string());
                        }
                        for widget in widgets {
                            if !matcher.matches(&widget) {
                                return Err(format!("Widget does not match expected criteria"));
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to find widgets: {}", e)),
                }
            } else {
                Err("Advanced testing not enabled".to_string())
            }
        });
    }

    /// Simular interacciones avanzadas
    #[cfg(feature = "testing")]
    pub fn perform_interaction(&mut self, interaction: impl Into<Box<dyn Interaction>>) -> Result<(), String> {
        if let Some(app) = &mut self.advanced_app {
            let tester = WidgetTester::new(app);
            tester.perform(interaction.into())
        } else {
            Err("Advanced testing not enabled".to_string())
        }
    }
}

/// Macros de testing para widgets (compatibilidad hacia atrás + nuevas capacidades)
#[macro_export]
macro_rules! widget_test {
    ($name:ident, $test:block) => {
        #[test]
        fn $name() {
            let mut runner = WidgetTestRunner::new();
            $test
            runner.run_assertions().expect("Widget test failed");
        }
    };
}

#[macro_export]
macro_rules! advanced_widget_test {
    ($name:ident, $test:block) => {
        #[test]
        fn $name() {
            let mut runner = WidgetTestRunner::with_advanced_testing();
            $test
            runner.run_assertions().expect("Advanced widget test failed");
        }
    };
}

macro_rules! simulate_event {
    ($runner:expr, $widget:expr, $event:expr) => {
        $runner.simulator().simulate_event($widget, $event)
            .expect(&format!("Failed to simulate event on widget '{}'", $widget));
    };
}

macro_rules! expect_property {
    ($runner:expr, $widget:expr, $property:expr, $value:expr) => {
        $runner.expect_property($widget, $property, serde_json::json!($value));
    };
}

macro_rules! expect_widget {
    ($runner:expr, $finder:expr, $matcher:expr) => {
        #[cfg(feature = "testing")]
        $runner.expect_widget($finder, $matcher);
    };
}

macro_rules! perform_interaction {
    ($runner:expr, $interaction:expr) => {
        #[cfg(feature = "testing")]
        $runner.perform_interaction($interaction)
            .expect("Failed to perform interaction");
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_click() {
        let mut runner = WidgetTestRunner::new();
        // Crear un botón
        let button = runner.simulator().create_widget("submit_button");
        button.set_property("text", serde_json::Value::String("Submit".to_string()));

        // Simular click
        runner.simulator().simulate_event("submit_button", WidgetEvent::Click)
            .expect("Failed to simulate event");

        // Verificar que el botón fue clickeado
        runner.expect_property("submit_button", "clicked", serde_json::Value::Bool(true));
        runner.run_assertions().expect("Widget test failed");
    }

    #[test]
    fn test_input_field() {
        let mut runner = WidgetTestRunner::new();
        // Crear un input field
        let input = runner.simulator().create_widget("username_input");

        // Simular input
        runner.simulator().simulate_event("username_input", WidgetEvent::Input("testuser".to_string()))
            .expect("Failed to simulate event");

        // Verificar el valor
        runner.expect_property("username_input", "value", serde_json::Value::String("testuser".to_string()));
        runner.run_assertions().expect("Widget test failed");
    }

    #[test]
    fn test_focus_blur() {
        let mut runner = WidgetTestRunner::new();
        // Crear un input field
        let input = runner.simulator().create_widget("email_input");

        // Simular focus
        runner.simulator().simulate_event("email_input", WidgetEvent::Focus)
            .expect("Failed to simulate event");

        // Verificar focus
        runner.expect_property("email_input", "focused", serde_json::Value::Bool(true));

        // Simular blur
        runner.simulator().simulate_event("email_input", WidgetEvent::Blur)
            .expect("Failed to simulate event");

        // Verificar blur
        runner.expect_property("email_input", "focused", serde_json::Value::Bool(false));
        runner.run_assertions().expect("Widget test failed");
    }

    #[test]
    fn test_event_logging() {
        let mut runner = WidgetTestRunner::new();
        // Crear un widget
        let _widget = runner.simulator().create_widget("test_widget");

        // Simular varios eventos
        runner.simulator().simulate_event("test_widget", WidgetEvent::Click)
            .expect("Failed to simulate event");
        runner.simulator().simulate_event("test_widget", WidgetEvent::Hover)
            .expect("Failed to simulate event");
        runner.simulator().simulate_event("test_widget", WidgetEvent::KeyPress("Enter".to_string()))
            .expect("Failed to simulate event");

        // Verificar log de eventos
        let log = runner.simulator().get_event_log();
        assert_eq!(log.len(), 3);
        assert_eq!(log[0], ("test_widget".to_string(), WidgetEvent::Click));
        assert_eq!(log[1], ("test_widget".to_string(), WidgetEvent::Hover));
        assert_eq!(log[2], ("test_widget".to_string(), WidgetEvent::KeyPress("Enter".to_string())));
    }

    // Tests avanzados usando el nuevo framework
    #[cfg(feature = "testing")]
    #[test]
    fn test_advanced_button_interaction() {
        let mut runner = WidgetTestRunner::with_advanced_testing();
        // Este test requiere que el framework avanzado esté completamente integrado
        // Por ahora, solo verificamos que se puede crear el runner avanzado
        assert!(runner.advanced_app().is_some());
    }
}