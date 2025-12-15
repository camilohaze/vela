//! iOS Event Bridge
//!
//! This module handles iOS touch events, gestures, and user interactions,
//! translating them into Vela events for the reactive system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Event bridge for iOS touch and gesture handling
pub struct VelaEventBridge {
    /// Event handlers registry
    event_handlers: HashMap<String, Vec<Box<dyn Fn(&VelaEvent) + Send + Sync>>>,
    /// Gesture recognizers
    gesture_recognizers: Vec<GestureRecognizer>,
    /// Touch state tracking
    touch_state: TouchState,
    /// Event queue for processing
    event_queue: Arc<Mutex<Vec<VelaEvent>>>,
}

impl VelaEventBridge {
    /// Create a new event bridge
    pub fn new() -> Self {
        Self {
            event_handlers: HashMap::new(),
            gesture_recognizers: Vec::new(),
            touch_state: TouchState::new(),
            event_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register an event handler for a widget
    pub fn register_handler<F>(&mut self, widget_id: &str, handler: F)
    where
        F: Fn(&VelaEvent) + Send + Sync + 'static,
    {
        self.event_handlers
            .entry(widget_id.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    /// Handle iOS touch event
    pub fn handle_touch_event(&mut self, touch_event: IOSTouchEvent) {
        let vela_event = self.translate_touch_event(touch_event);

        // Queue event for processing
        if let Ok(mut queue) = self.event_queue.lock() {
            queue.push(vela_event);
        }
    }

    /// Handle iOS gesture event
    pub fn handle_gesture_event(&mut self, gesture_event: IOSGestureEvent) {
        let vela_event = self.translate_gesture_event(gesture_event);

        // Queue event for processing
        if let Ok(mut queue) = self.event_queue.lock() {
            queue.push(vela_event);
        }
    }

    /// Process queued events and dispatch to handlers
    pub fn process_events(&mut self) {
        let events = {
            let mut queue = self.event_queue.lock().unwrap();
            std::mem::take(&mut *queue)
        };

        for event in events {
            self.dispatch_event(&event);
        }
    }

    /// Add a gesture recognizer
    pub fn add_gesture_recognizer(&mut self, recognizer: GestureRecognizer) {
        self.gesture_recognizers.push(recognizer);
    }

    /// Translate iOS touch event to Vela event
    fn translate_touch_event(&self, ios_event: IOSTouchEvent) -> VelaEvent {
        match ios_event.phase {
            TouchPhase::Began => VelaEvent::PointerDown {
                x: ios_event.location.x,
                y: ios_event.location.y,
                pointer_id: ios_event.touch_id,
                target: ios_event.target_widget.clone(),
            },
            TouchPhase::Moved => VelaEvent::PointerMove {
                x: ios_event.location.x,
                y: ios_event.location.y,
                pointer_id: ios_event.touch_id,
                target: ios_event.target_widget.clone(),
            },
            TouchPhase::Ended => VelaEvent::PointerUp {
                x: ios_event.location.x,
                y: ios_event.location.y,
                pointer_id: ios_event.touch_id,
                target: ios_event.target_widget.clone(),
            },
            TouchPhase::Cancelled => VelaEvent::PointerCancel {
                pointer_id: ios_event.touch_id,
                target: ios_event.target_widget.clone(),
            },
        }
    }

    /// Translate iOS gesture event to Vela event
    fn translate_gesture_event(&self, ios_event: IOSGestureEvent) -> VelaEvent {
        match ios_event.gesture_type {
            GestureType::Tap => VelaEvent::Tap {
                x: ios_event.location.x,
                y: ios_event.location.y,
                target: ios_event.target_widget.clone(),
            },
            GestureType::DoubleTap => VelaEvent::DoubleTap {
                x: ios_event.location.x,
                y: ios_event.location.y,
                target: ios_event.target_widget.clone(),
            },
            GestureType::LongPress => VelaEvent::LongPress {
                x: ios_event.location.x,
                y: ios_event.location.y,
                target: ios_event.target_widget.clone(),
            },
            GestureType::Pan => VelaEvent::Pan {
                delta_x: ios_event.delta_x,
                delta_y: ios_event.delta_y,
                velocity_x: ios_event.velocity_x,
                velocity_y: ios_event.velocity_y,
                target: ios_event.target_widget.clone(),
            },
            GestureType::Pinch => VelaEvent::Pinch {
                scale: ios_event.scale,
                velocity: ios_event.velocity,
                target: ios_event.target_widget.clone(),
            },
            GestureType::Rotate => VelaEvent::Rotate {
                rotation: ios_event.rotation,
                velocity: ios_event.velocity,
                target: ios_event.target_widget.clone(),
            },
            GestureType::Swipe => VelaEvent::Swipe {
                direction: ios_event.direction,
                velocity: ios_event.velocity,
                target: ios_event.target_widget.clone(),
            },
        }
    }

    /// Dispatch event to registered handlers
    fn dispatch_event(&self, event: &VelaEvent) {
        let target = event.target();

        if let Some(handlers) = self.event_handlers.get(target) {
            for handler in handlers {
                handler(event);
            }
        }

        // Also dispatch to global handlers (empty string key)
        if let Some(handlers) = self.event_handlers.get("") {
            for handler in handlers {
                handler(event);
            }
        }
    }
}

/// Touch state tracking
pub struct TouchState {
    active_touches: HashMap<u32, TouchInfo>,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            active_touches: HashMap::new(),
        }
    }

    pub fn track_touch(&mut self, touch_id: u32, info: TouchInfo) {
        self.active_touches.insert(touch_id, info);
    }

    pub fn remove_touch(&mut self, touch_id: u32) {
        self.active_touches.remove(&touch_id);
    }

    pub fn get_touch(&self, touch_id: u32) -> Option<&TouchInfo> {
        self.active_touches.get(&touch_id)
    }
}

/// Touch information
#[derive(Clone, Debug)]
pub struct TouchInfo {
    pub start_location: Point,
    pub current_location: Point,
    pub timestamp: f64,
}

/// Point structure
#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// iOS touch event structure
#[derive(Clone, Debug)]
pub struct IOSTouchEvent {
    pub touch_id: u32,
    pub phase: TouchPhase,
    pub location: Point,
    pub target_widget: String,
    pub timestamp: f64,
}

/// Touch phase
#[derive(Clone, Debug)]
pub enum TouchPhase {
    Began,
    Moved,
    Ended,
    Cancelled,
}

/// iOS gesture event structure
#[derive(Clone, Debug)]
pub struct IOSGestureEvent {
    pub gesture_type: GestureType,
    pub location: Point,
    pub target_widget: String,
    pub delta_x: f32,
    pub delta_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub scale: f32,
    pub velocity: f32,
    pub rotation: f32,
    pub direction: SwipeDirection,
}

/// Gesture types
#[derive(Clone, Debug)]
pub enum GestureType {
    Tap,
    DoubleTap,
    LongPress,
    Pan,
    Pinch,
    Rotate,
    Swipe,
}

/// Swipe direction
#[derive(Clone, Debug)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Gesture recognizer configuration
#[derive(Clone, Debug)]
pub struct GestureRecognizer {
    pub gesture_type: GestureType,
    pub minimum_touches: u32,
    pub maximum_touches: u32,
    pub minimum_press_duration: f64,
    pub allowable_movement: f32,
}

/// Vela event types
#[derive(Clone, Debug)]
pub enum VelaEvent {
    PointerDown {
        x: f32,
        y: f32,
        pointer_id: u32,
        target: String,
    },
    PointerMove {
        x: f32,
        y: f32,
        pointer_id: u32,
        target: String,
    },
    PointerUp {
        x: f32,
        y: f32,
        pointer_id: u32,
        target: String,
    },
    PointerCancel {
        pointer_id: u32,
        target: String,
    },
    Tap {
        x: f32,
        y: f32,
        target: String,
    },
    DoubleTap {
        x: f32,
        y: f32,
        target: String,
    },
    LongPress {
        x: f32,
        y: f32,
        target: String,
    },
    Pan {
        delta_x: f32,
        delta_y: f32,
        velocity_x: f32,
        velocity_y: f32,
        target: String,
    },
    Pinch {
        scale: f32,
        velocity: f32,
        target: String,
    },
    Rotate {
        rotation: f32,
        velocity: f32,
        target: String,
    },
    Swipe {
        direction: SwipeDirection,
        velocity: f32,
        target: String,
    },
}

impl VelaEvent {
    /// Get the target widget ID
    pub fn target(&self) -> &str {
        match self {
            VelaEvent::PointerDown { target, .. } => target,
            VelaEvent::PointerMove { target, .. } => target,
            VelaEvent::PointerUp { target, .. } => target,
            VelaEvent::PointerCancel { target, .. } => target,
            VelaEvent::Tap { target, .. } => target,
            VelaEvent::DoubleTap { target, .. } => target,
            VelaEvent::LongPress { target, .. } => target,
            VelaEvent::Pan { target, .. } => target,
            VelaEvent::Pinch { target, .. } => target,
            VelaEvent::Rotate { target, .. } => target,
            VelaEvent::Swipe { target, .. } => target,
        }
    }
}