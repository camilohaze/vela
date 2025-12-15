//! Gesture Recognition System
//!
//! This module provides comprehensive gesture recognition for touch and pointer interactions.
//! Supports tap, drag, pinch, rotate, swipe, and long press gestures with advanced
//! composition and competition handling.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::ui::animated::SignalValue;

/// Core gesture event types
#[derive(Debug, Clone)]
pub enum GestureEvent {
    Tap,
    DoubleTap,
    LongPress,
    DragStart(DragStartDetails),
    DragUpdate(DragUpdateDetails),
    DragEnd(DragEndDetails),
    PinchStart(PinchStartDetails),
    PinchUpdate(PinchUpdateDetails),
    PinchEnd(PinchEndDetails),
    RotateStart(RotateStartDetails),
    RotateUpdate(RotateUpdateDetails),
    RotateEnd(RotateEndDetails),
    Swipe(SwipeDetails),
}

/// Point in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn angle(&self, other: &Point) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dy.atan2(dx)
    }
}

/// Velocity vector
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Pointer event from the platform
#[derive(Debug, Clone)]
pub struct PointerEvent {
    pub pointer_id: u32,
    pub position: Point,
    pub timestamp: Instant,
    pub event_type: PointerEventType,
}

#[derive(Debug, Clone)]
pub enum PointerEventType {
    Down,
    Move,
    Up,
    Cancel,
}

/// Gesture configuration parameters
#[derive(Debug, Clone)]
pub struct GestureConfig {
    pub tap_timeout: Duration,
    pub double_tap_timeout: Duration,
    pub long_press_timeout: Duration,
    pub min_drag_distance: f32,
    pub min_swipe_velocity: f32,
    pub max_swipe_angle: f32,
    pub pinch_slop: f32,
    pub rotate_slop: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_timeout: Duration::from_millis(300),
            double_tap_timeout: Duration::from_millis(300),
            long_press_timeout: Duration::from_millis(500),
            min_drag_distance: 10.0,
            min_swipe_velocity: 500.0,
            max_swipe_angle: 45.0_f32.to_radians(),
            pinch_slop: 20.0,
            rotate_slop: 10.0_f32.to_radians(),
        }
    }
}

/// Gesture event details
#[derive(Debug, Clone)]
pub struct DragStartDetails {
    pub global_position: Point,
    pub local_position: Point,
}

#[derive(Debug, Clone)]
pub struct DragUpdateDetails {
    pub global_position: Point,
    pub local_position: Point,
    pub delta: Point,
    pub primary_delta: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct DragEndDetails {
    pub velocity: Velocity,
    pub primary_velocity: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct PinchStartDetails {
    pub focal_point: Point,
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct PinchUpdateDetails {
    pub focal_point: Point,
    pub scale: f32,
    pub rotation: f32,
}

#[derive(Debug, Clone)]
pub struct PinchEndDetails {
    pub velocity: f32,
}

#[derive(Debug, Clone)]
pub struct RotateStartDetails {
    pub focal_point: Point,
    pub rotation: f32,
}

#[derive(Debug, Clone)]
pub struct RotateUpdateDetails {
    pub focal_point: Point,
    pub rotation: f32,
}

#[derive(Debug, Clone)]
pub struct RotateEndDetails {
    pub velocity: f32,
}

#[derive(Debug, Clone)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct SwipeDetails {
    pub direction: SwipeDirection,
    pub velocity: Velocity,
}

/// Gesture recognizer trait
pub trait GestureRecognizer: Send + Sync {
    fn add_pointer(&mut self, event: PointerEvent);
    fn handle_event(&mut self, event: PointerEvent) -> Option<GestureEvent>;
    fn reset(&mut self);
    fn accept_gesture(&mut self, pointer_id: u32) -> bool;
    fn reject_gesture(&mut self, pointer_id: u32);
}

/// Tap gesture recognizer
pub struct TapGestureRecognizer {
    config: GestureConfig,
    state: TapState,
    tap_count: u32,
    last_tap_time: Option<Instant>,
    last_tap_position: Option<Point>,
    pending_timeout: Option<Instant>,
}

#[derive(Debug, Clone)]
enum TapState {
    Ready,
    WaitingForDoubleTap,
    WaitingForTimeout,
    DoubleTapDetected,
}

impl TapGestureRecognizer {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            state: TapState::Ready,
            tap_count: 0,
            last_tap_time: None,
            last_tap_position: None,
            pending_timeout: None,
        }
    }
}

impl GestureRecognizer for TapGestureRecognizer {
    fn add_pointer(&mut self, event: PointerEvent) {
        match event.event_type {
            PointerEventType::Down => {
                self.tap_count += 1;
                self.last_tap_position = Some(event.position);
                self.last_tap_time = Some(event.timestamp);
                self.pending_timeout = Some(event.timestamp + self.config.tap_timeout);
                self.state = TapState::WaitingForTimeout;
            }
            PointerEventType::Move => {
                // Check if moved too far for tap
                if let Some(last_pos) = self.last_tap_position {
                    if event.position.distance(&last_pos) > self.config.min_drag_distance {
                        self.reset();
                    }
                }
            }
            PointerEventType::Up => {
                if let Some(timeout) = self.pending_timeout {
                    if event.timestamp <= timeout {
                        match self.state {
                            TapState::WaitingForTimeout => {
                                if self.tap_count == 1 {
                                    self.state = TapState::WaitingForDoubleTap;
                                    self.pending_timeout = Some(event.timestamp + self.config.double_tap_timeout);
                                } else if self.tap_count == 2 {
                                    // Double tap detected - will generate event in handle_event
                                    self.state = TapState::DoubleTapDetected;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            PointerEventType::Cancel => {
                self.reset();
            }
        }
    }

    fn handle_event(&mut self, event: PointerEvent) -> Option<GestureEvent> {
        // Check for immediate events first
        match self.state {
            TapState::DoubleTapDetected => {
                self.reset();
                return Some(GestureEvent::DoubleTap);
            }
            _ => {}
        }

        // Check timeouts
        if let Some(timeout) = self.pending_timeout {
            if event.timestamp > timeout {
                match self.state {
                    TapState::WaitingForTimeout => {
                        if self.tap_count == 1 {
                            self.reset();
                            return Some(GestureEvent::Tap);
                        }
                    }
                    TapState::WaitingForDoubleTap => {
                        self.reset();
                        return Some(GestureEvent::Tap);
                    }
                    _ => {}
                }
            }
        }

        // Check for long press
        if let Some(last_time) = self.last_tap_time {
            if event.timestamp.duration_since(last_time) >= self.config.long_press_timeout
                && matches!(self.state, TapState::WaitingForTimeout) {
                self.reset();
                return Some(GestureEvent::LongPress);
            }
        }

        None
    }

    fn reset(&mut self) {
        self.state = TapState::Ready;
        self.tap_count = 0;
        self.last_tap_time = None;
        self.last_tap_position = None;
        self.pending_timeout = None;
    }

    fn accept_gesture(&mut self, _pointer_id: u32) -> bool {
        true
    }

    fn reject_gesture(&mut self, _pointer_id: u32) {
        self.reset();
    }
}

/// Drag gesture recognizer
pub struct DragGestureRecognizer {
    config: GestureConfig,
    state: DragState,
    initial_position: Point,
    current_position: Point,
    pointer_id: Option<u32>,
    drag_started: bool,
}

#[derive(Debug, Clone)]
enum DragState {
    Ready,
    Accepted,
    Rejected,
    Ended,
}

impl DragGestureRecognizer {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            state: DragState::Ready,
            initial_position: Point::new(0.0, 0.0),
            current_position: Point::new(0.0, 0.0),
            pointer_id: None,
            drag_started: false,
        }
    }
}

impl GestureRecognizer for DragGestureRecognizer {
    fn add_pointer(&mut self, event: PointerEvent) {
        match event.event_type {
            PointerEventType::Down => {
                self.initial_position = event.position;
                self.current_position = event.position;
                self.pointer_id = Some(event.pointer_id);
                self.state = DragState::Ready;
            }
            PointerEventType::Move => {
                if self.pointer_id == Some(event.pointer_id) {
                    self.current_position = event.position;

                    match self.state {
                        DragState::Ready => {
                            if self.current_position.distance(&self.initial_position) >= self.config.min_drag_distance {
                                self.state = DragState::Accepted;
                            }
                        }
                        DragState::Accepted => {
                            // Continue tracking
                        }
                        DragState::Rejected => {
                            // Do nothing
                        }
                        DragState::Ended => {
                            // Gesture already ended, do nothing
                        }
                    }
                }
            }
            PointerEventType::Up | PointerEventType::Cancel => {
                if self.pointer_id == Some(event.pointer_id) {
                    match self.state {
                        DragState::Accepted => {
                            self.state = DragState::Ended;
                        }
                        _ => {
                            self.reset();
                        }
                    }
                }
            }
        }
    }

    fn handle_event(&mut self, _event: PointerEvent) -> Option<GestureEvent> {
        match self.state {
            DragState::Accepted => {
                if !self.drag_started {
                    // First time accepted - generate DragStart
                    self.drag_started = true;
                    Some(GestureEvent::DragStart(DragStartDetails {
                        global_position: self.initial_position,
                        local_position: self.initial_position, // TODO: Transform to local coordinates
                    }))
                } else {
                    // Continuing drag - generate DragUpdate
                    let delta = Point::new(
                        self.current_position.x - self.initial_position.x,
                        self.current_position.y - self.initial_position.y,
                    );

                    Some(GestureEvent::DragUpdate(DragUpdateDetails {
                        global_position: self.current_position,
                        local_position: self.current_position, // TODO: Transform to local coordinates
                        delta,
                        primary_delta: None,
                    }))
                }
            }
            DragState::Ended => {
                let delta = Point::new(
                    self.current_position.x - self.initial_position.x,
                    self.current_position.y - self.initial_position.y,
                );

                self.reset(); // Reset after generating the event

                Some(GestureEvent::DragEnd(DragEndDetails {
                    velocity: Velocity::new(0.0, 0.0), // TODO: Calculate velocity
                    primary_velocity: None,
                }))
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.state = DragState::Ready;
        self.pointer_id = None;
        self.drag_started = false;
    }

    fn accept_gesture(&mut self, pointer_id: u32) -> bool {
        if self.pointer_id == Some(pointer_id) {
            self.state = DragState::Accepted;
            true
        } else {
            false
        }
    }

    fn reject_gesture(&mut self, pointer_id: u32) {
        if self.pointer_id == Some(pointer_id) {
            self.state = DragState::Rejected;
        }
    }
}

/// Pinch gesture recognizer
pub struct PinchGestureRecognizer {
    config: GestureConfig,
    state: PinchState,
    pointers: HashMap<u32, Point>,
    initial_distance: f32,
    current_distance: f32,
    scale: f32,
    focal_point: Point,
    pinch_started: bool,
}

#[derive(Debug, Clone)]
enum PinchState {
    Ready,
    Accepted,
    Rejected,
}

impl PinchGestureRecognizer {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            state: PinchState::Ready,
            pointers: HashMap::new(),
            initial_distance: 0.0,
            current_distance: 0.0,
            scale: 1.0,
            focal_point: Point::new(0.0, 0.0),
            pinch_started: false,
        }
    }

    fn update_focal_point(&mut self) {
        if self.pointers.len() >= 2 {
            let points: Vec<&Point> = self.pointers.values().collect();
            let mut x_sum = 0.0;
            let mut y_sum = 0.0;

            for point in &points {
                x_sum += point.x;
                y_sum += point.y;
            }

            self.focal_point = Point::new(
                x_sum / points.len() as f32,
                y_sum / points.len() as f32,
            );
        }
    }

    fn update_distance(&mut self) {
        if self.pointers.len() >= 2 {
            let points: Vec<&Point> = self.pointers.values().collect();
            self.current_distance = points[0].distance(points[1]);
        }
    }
}

impl GestureRecognizer for PinchGestureRecognizer {
    fn add_pointer(&mut self, event: PointerEvent) {
        match event.event_type {
            PointerEventType::Down => {
                self.pointers.insert(event.pointer_id, event.position);
                self.update_focal_point();

                if self.pointers.len() == 2 {
                    self.update_distance();
                    self.initial_distance = self.current_distance;
                    self.scale = 1.0;
                    self.state = PinchState::Ready;
                }
            }
            PointerEventType::Move => {
                if let Some(pos) = self.pointers.get_mut(&event.pointer_id) {
                    *pos = event.position;
                    self.update_focal_point();
                    self.update_distance();

                    match self.state {
                        PinchState::Ready => {
                            let scale_diff = (self.current_distance / self.initial_distance - 1.0).abs();
                            if scale_diff >= self.config.pinch_slop / 100.0 {
                                self.state = PinchState::Accepted;
                            }
                        }
                        PinchState::Accepted => {
                            self.scale = self.current_distance / self.initial_distance;
                        }
                        PinchState::Rejected => {
                            // Do nothing
                        }
                    }
                }
            }
            PointerEventType::Up | PointerEventType::Cancel => {
                self.pointers.remove(&event.pointer_id);
                if self.pointers.len() < 2 {
                    self.reset();
                }
            }
        }
    }

    fn handle_event(&mut self, _event: PointerEvent) -> Option<GestureEvent> {
        match self.state {
            PinchState::Accepted => {
                if !self.pinch_started {
                    // First time accepted - generate PinchStart
                    self.pinch_started = true;
                    Some(GestureEvent::PinchStart(PinchStartDetails {
                        focal_point: self.focal_point,
                        scale: self.scale,
                    }))
                } else {
                    // Continuing pinch - generate PinchUpdate
                    Some(GestureEvent::PinchUpdate(PinchUpdateDetails {
                        focal_point: self.focal_point,
                        scale: self.scale,
                        rotation: 0.0, // TODO: Implement rotation
                    }))
                }
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.state = PinchState::Ready;
        self.pointers.clear();
        self.initial_distance = 0.0;
        self.current_distance = 0.0;
        self.scale = 1.0;
        self.pinch_started = false;
    }

    fn accept_gesture(&mut self, _pointer_id: u32) -> bool {
        if self.pointers.len() >= 2 {
            self.state = PinchState::Accepted;
            true
        } else {
            false
        }
    }

    fn reject_gesture(&mut self, _pointer_id: u32) {
        self.state = PinchState::Rejected;
    }
}

/// Rotate gesture recognizer
pub struct RotateGestureRecognizer {
    config: GestureConfig,
    state: RotateState,
    pointers: HashMap<u32, Point>,
    initial_angle: f32,
    current_angle: f32,
    rotation: f32,
    focal_point: Point,
}

#[derive(Debug, Clone)]
enum RotateState {
    Ready,
    Accepted,
    Rejected,
}

impl RotateGestureRecognizer {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            state: RotateState::Ready,
            pointers: HashMap::new(),
            initial_angle: 0.0,
            current_angle: 0.0,
            rotation: 0.0,
            focal_point: Point::new(0.0, 0.0),
        }
    }

    fn update_focal_point(&mut self) {
        if self.pointers.len() >= 2 {
            let points: Vec<&Point> = self.pointers.values().collect();
            let mut x_sum = 0.0;
            let mut y_sum = 0.0;

            for point in &points {
                x_sum += point.x;
                y_sum += point.y;
            }

            self.focal_point = Point::new(
                x_sum / points.len() as f32,
                y_sum / points.len() as f32,
            );
        }
    }

    fn update_angle(&mut self) {
        if self.pointers.len() >= 2 {
            let points: Vec<&Point> = self.pointers.values().collect();
            self.current_angle = points[0].angle(points[1]);
        }
    }
}

impl GestureRecognizer for RotateGestureRecognizer {
    fn add_pointer(&mut self, event: PointerEvent) {
        match event.event_type {
            PointerEventType::Down => {
                self.pointers.insert(event.pointer_id, event.position);
                self.update_focal_point();

                if self.pointers.len() == 2 {
                    self.update_angle();
                    self.initial_angle = self.current_angle;
                    self.rotation = 0.0;
                    self.state = RotateState::Ready;
                }
            }
            PointerEventType::Move => {
                if let Some(pos) = self.pointers.get_mut(&event.pointer_id) {
                    *pos = event.position;
                    self.update_focal_point();
                    self.update_angle();

                    match self.state {
                        RotateState::Ready => {
                            let angle_diff = (self.current_angle - self.initial_angle).abs();
                            if angle_diff >= self.config.rotate_slop {
                                self.state = RotateState::Accepted;
                            }
                        }
                        RotateState::Accepted => {
                            self.rotation = self.current_angle - self.initial_angle;
                        }
                        RotateState::Rejected => {
                            // Do nothing
                        }
                    }
                }
            }
            PointerEventType::Up | PointerEventType::Cancel => {
                self.pointers.remove(&event.pointer_id);
                if self.pointers.len() < 2 {
                    self.reset();
                }
            }
        }
    }

    fn handle_event(&mut self, _event: PointerEvent) -> Option<GestureEvent> {
        match self.state {
            RotateState::Accepted => {
                Some(GestureEvent::RotateUpdate(RotateUpdateDetails {
                    focal_point: self.focal_point,
                    rotation: self.rotation,
                }))
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.state = RotateState::Ready;
        self.pointers.clear();
        self.initial_angle = 0.0;
        self.current_angle = 0.0;
        self.rotation = 0.0;
    }

    fn accept_gesture(&mut self, _pointer_id: u32) -> bool {
        if self.pointers.len() >= 2 {
            self.state = RotateState::Accepted;
            true
        } else {
            false
        }
    }

    fn reject_gesture(&mut self, _pointer_id: u32) {
        self.state = RotateState::Rejected;
    }
}

/// Swipe gesture recognizer
pub struct SwipeGestureRecognizer {
    config: GestureConfig,
    state: SwipeState,
    initial_position: Point,
    current_position: Point,
    start_time: Option<Instant>,
    pointer_id: Option<u32>,
}

#[derive(Debug, Clone)]
enum SwipeState {
    Ready,
    Tracking,
    Completed,
}

impl SwipeGestureRecognizer {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            state: SwipeState::Ready,
            initial_position: Point::new(0.0, 0.0),
            current_position: Point::new(0.0, 0.0),
            start_time: None,
            pointer_id: None,
        }
    }

    fn detect_direction(&self) -> Option<SwipeDirection> {
        let delta_x = self.current_position.x - self.initial_position.x;
        let delta_y = self.current_position.y - self.initial_position.y;
        let abs_x = delta_x.abs();
        let abs_y = delta_y.abs();

        if abs_x > abs_y {
            if abs_x > self.config.min_drag_distance {
                if delta_x > 0.0 {
                    Some(SwipeDirection::Right)
                } else {
                    Some(SwipeDirection::Left)
                }
            } else {
                None
            }
        } else {
            if abs_y > self.config.min_drag_distance {
                if delta_y > 0.0 {
                    Some(SwipeDirection::Down)
                } else {
                    Some(SwipeDirection::Up)
                }
            } else {
                None
            }
        }
    }

    fn calculate_velocity(&self, duration: Duration) -> Velocity {
        let dt = duration.as_secs_f32();
        if dt > 0.0 {
            Velocity::new(
                (self.current_position.x - self.initial_position.x) / dt,
                (self.current_position.y - self.initial_position.y) / dt,
            )
        } else {
            Velocity::new(0.0, 0.0)
        }
    }
}

impl GestureRecognizer for SwipeGestureRecognizer {
    fn add_pointer(&mut self, event: PointerEvent) {
        match event.event_type {
            PointerEventType::Down => {
                self.initial_position = event.position;
                self.current_position = event.position;
                self.start_time = Some(event.timestamp);
                self.pointer_id = Some(event.pointer_id);
                self.state = SwipeState::Tracking;
            }
            PointerEventType::Move => {
                if self.pointer_id == Some(event.pointer_id) {
                    self.current_position = event.position;
                }
            }
            PointerEventType::Up => {
                if self.pointer_id == Some(event.pointer_id) {
                    if let Some(start_time) = self.start_time {
                        let duration = event.timestamp.duration_since(start_time);
                        let velocity = self.calculate_velocity(duration);

                        if velocity.magnitude() >= self.config.min_swipe_velocity {
                            if let Some(direction) = self.detect_direction() {
                                self.state = SwipeState::Completed;
                            }
                        }
                    }
                }
            }
            PointerEventType::Cancel => {
                self.reset();
            }
        }
    }

    fn handle_event(&mut self, event: PointerEvent) -> Option<GestureEvent> {
        if matches!(self.state, SwipeState::Completed) {
            if let Some(start_time) = self.start_time {
                let duration = event.timestamp.duration_since(start_time);
                let velocity = self.calculate_velocity(duration);

                if let Some(direction) = self.detect_direction() {
                    self.reset();
                    return Some(GestureEvent::Swipe(SwipeDetails {
                        direction,
                        velocity,
                    }));
                }
            }
        }
        None
    }

    fn reset(&mut self) {
        self.state = SwipeState::Ready;
        self.start_time = None;
        self.pointer_id = None;
    }

    fn accept_gesture(&mut self, pointer_id: u32) -> bool {
        self.pointer_id == Some(pointer_id)
    }

    fn reject_gesture(&mut self, pointer_id: u32) {
        if self.pointer_id == Some(pointer_id) {
            self.reset();
        }
    }
}

/// Gesture arena for managing competing gestures
pub type GestureArenaMemberId = u64;

pub struct GestureArenaMember {
    pub recognizer: Box<dyn GestureRecognizer>,
    pub is_accepted: bool,
}

pub struct GestureArena {
    members: HashMap<GestureArenaMemberId, GestureArenaMember>,
    next_id: GestureArenaMemberId,
    winner: Option<GestureArenaMemberId>,
}

impl GestureArena {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
            next_id: 0,
            winner: None,
        }
    }

    pub fn add(&mut self, recognizer: Box<dyn GestureRecognizer>) -> GestureArenaMemberId {
        let id = self.next_id;
        self.next_id += 1;

        self.members.insert(id, GestureArenaMember {
            recognizer,
            is_accepted: false,
        });

        id
    }

    pub fn resolve(&mut self, winner: GestureArenaMemberId) {
        self.winner = Some(winner);

        // Accept winner and reject others
        for (id, member) in &mut self.members {
            if *id == winner {
                member.is_accepted = true;
            } else {
                // Reject other members
                member.recognizer.reject_gesture(0); // TODO: Pass correct pointer ID
            }
        }
    }

    pub fn sweep(&mut self, _deadline: Instant) {
        // Remove completed gestures
        self.members.retain(|_, member| !matches!(member.recognizer.handle_event(PointerEvent {
            pointer_id: 0,
            position: Point::new(0.0, 0.0),
            timestamp: Instant::now(),
            event_type: PointerEventType::Cancel,
        }), Some(_)));
    }

    pub fn process_event(&mut self, event: PointerEvent) -> Vec<GestureEvent> {
        let mut events = Vec::new();

        for member in self.members.values_mut() {
            member.recognizer.add_pointer(event.clone());

            if let Some(gesture_event) = member.recognizer.handle_event(event.clone()) {
                events.push(gesture_event);
            }
        }

        events
    }
}

/// Main gesture detector widget
pub struct GestureDetector {
    pub child: Box<dyn crate::ui::Widget>,
    pub config: GestureConfig,
    pub arena: GestureArena,

    // Callbacks
    pub on_tap: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_double_tap: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_long_press: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_drag_start: Option<Arc<dyn Fn(DragStartDetails) + Send + Sync>>,
    pub on_drag_update: Option<Arc<dyn Fn(DragUpdateDetails) + Send + Sync>>,
    pub on_drag_end: Option<Arc<dyn Fn(DragEndDetails) + Send + Sync>>,
    pub on_pinch_start: Option<Arc<dyn Fn(PinchStartDetails) + Send + Sync>>,
    pub on_pinch_update: Option<Arc<dyn Fn(PinchUpdateDetails) + Send + Sync>>,
    pub on_pinch_end: Option<Arc<dyn Fn(PinchEndDetails) + Send + Sync>>,
    pub on_rotate_start: Option<Arc<dyn Fn(RotateStartDetails) + Send + Sync>>,
    pub on_rotate_update: Option<Arc<dyn Fn(RotateUpdateDetails) + Send + Sync>>,
    pub on_rotate_end: Option<Arc<dyn Fn(RotateEndDetails) + Send + Sync>>,
    pub on_swipe: Option<Arc<dyn Fn(SwipeDetails) + Send + Sync>>,
}

impl GestureDetector {
    pub fn new(child: Box<dyn crate::ui::Widget>) -> Self {
        Self {
            child,
            config: GestureConfig::default(),
            arena: GestureArena::new(),
            on_tap: None,
            on_double_tap: None,
            on_long_press: None,
            on_drag_start: None,
            on_drag_update: None,
            on_drag_end: None,
            on_pinch_start: None,
            on_pinch_update: None,
            on_pinch_end: None,
            on_rotate_start: None,
            on_rotate_update: None,
            on_rotate_end: None,
            on_swipe: None,
        }
    }

    pub fn with_config(mut self, config: GestureConfig) -> Self {
        self.config = config;
        self
    }

    pub fn on_tap<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_tap = Some(Arc::new(callback));
        self
    }

    pub fn on_double_tap<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_double_tap = Some(Arc::new(callback));
        self
    }

    pub fn on_long_press<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_long_press = Some(Arc::new(callback));
        self
    }

    pub fn on_drag_start<F>(mut self, callback: F) -> Self
    where
        F: Fn(DragStartDetails) + Send + Sync + 'static,
    {
        self.on_drag_start = Some(Arc::new(callback));
        self
    }

    pub fn on_drag_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(DragUpdateDetails) + Send + Sync + 'static,
    {
        self.on_drag_update = Some(Arc::new(callback));
        self
    }

    pub fn on_drag_end<F>(mut self, callback: F) -> Self
    where
        F: Fn(DragEndDetails) + Send + Sync + 'static,
    {
        self.on_drag_end = Some(Arc::new(callback));
        self
    }

    pub fn on_pinch_start<F>(mut self, callback: F) -> Self
    where
        F: Fn(PinchStartDetails) + Send + Sync + 'static,
    {
        self.on_pinch_start = Some(Arc::new(callback));
        self
    }

    pub fn on_pinch_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(PinchUpdateDetails) + Send + Sync + 'static,
    {
        self.on_pinch_update = Some(Arc::new(callback));
        self
    }

    pub fn on_pinch_end<F>(mut self, callback: F) -> Self
    where
        F: Fn(PinchEndDetails) + Send + Sync + 'static,
    {
        self.on_pinch_end = Some(Arc::new(callback));
        self
    }

    pub fn on_rotate_start<F>(mut self, callback: F) -> Self
    where
        F: Fn(RotateStartDetails) + Send + Sync + 'static,
    {
        self.on_rotate_start = Some(Arc::new(callback));
        self
    }

    pub fn on_rotate_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(RotateUpdateDetails) + Send + Sync + 'static,
    {
        self.on_rotate_update = Some(Arc::new(callback));
        self
    }

    pub fn on_rotate_end<F>(mut self, callback: F) -> Self
    where
        F: Fn(RotateEndDetails) + Send + Sync + 'static,
    {
        self.on_rotate_end = Some(Arc::new(callback));
        self
    }

    pub fn on_swipe<F>(mut self, callback: F) -> Self
    where
        F: Fn(SwipeDetails) + Send + Sync + 'static,
    {
        self.on_swipe = Some(Arc::new(callback));
        self
    }

    pub fn handle_pointer_event(&mut self, event: PointerEvent) {
        let gesture_events = self.arena.process_event(event);

        for gesture_event in gesture_events {
            self.dispatch_gesture_event(gesture_event);
        }
    }

    fn dispatch_gesture_event(&self, event: GestureEvent) {
        match event {
            GestureEvent::Tap => {
                if let Some(callback) = &self.on_tap {
                    callback();
                }
            }
            GestureEvent::DoubleTap => {
                if let Some(callback) = &self.on_double_tap {
                    callback();
                }
            }
            GestureEvent::LongPress => {
                if let Some(callback) = &self.on_long_press {
                    callback();
                }
            }
            GestureEvent::DragStart(details) => {
                if let Some(callback) = &self.on_drag_start {
                    callback(details);
                }
            }
            GestureEvent::DragUpdate(details) => {
                if let Some(callback) = &self.on_drag_update {
                    callback(details);
                }
            }
            GestureEvent::DragEnd(details) => {
                if let Some(callback) = &self.on_drag_end {
                    callback(details);
                }
            }
            GestureEvent::PinchStart(details) => {
                if let Some(callback) = &self.on_pinch_start {
                    callback(details);
                }
            }
            GestureEvent::PinchUpdate(details) => {
                if let Some(callback) = &self.on_pinch_update {
                    callback(details);
                }
            }
            GestureEvent::PinchEnd(details) => {
                if let Some(callback) = &self.on_pinch_end {
                    callback(details);
                }
            }
            GestureEvent::RotateStart(details) => {
                if let Some(callback) = &self.on_rotate_start {
                    callback(details);
                }
            }
            GestureEvent::RotateUpdate(details) => {
                if let Some(callback) = &self.on_rotate_update {
                    callback(details);
                }
            }
            GestureEvent::RotateEnd(details) => {
                if let Some(callback) = &self.on_rotate_end {
                    callback(details);
                }
            }
            GestureEvent::Swipe(details) => {
                if let Some(callback) = &self.on_swipe {
                    callback(details);
                }
            }
        }
    }

    pub fn add_gesture_recognizer(&mut self, recognizer: Box<dyn GestureRecognizer>) {
        self.arena.add(recognizer);
    }

    pub fn setup_default_gestures(&mut self) {
        self.add_gesture_recognizer(Box::new(TapGestureRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(DragGestureRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(PinchGestureRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(RotateGestureRecognizer::new(self.config.clone())));
        self.add_gesture_recognizer(Box::new(SwipeGestureRecognizer::new(self.config.clone())));
    }
}

/// Basic widget implementation for testing
pub struct BasicWidget {
    id: String,
}

impl BasicWidget {
    pub fn new() -> Self {
        Self {
            id: "basic_widget".to_string(),
        }
    }
}

impl crate::ui::Widget for BasicWidget {
    fn render(&self) -> String {
        format!("BasicWidget(id: {})", self.id)
    }

    fn update_property(&mut self, _property: &str, _value: SignalValue) {
        // Basic widget doesn't have animatable properties
    }
}

impl Default for GestureDetector {
    fn default() -> Self {
        let mut detector = Self::new(Box::new(BasicWidget::new()));
        detector.setup_default_gestures();
        detector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_gesture_config {
        use super::*;

        #[test]
        fn test_default_config() {
            let config = GestureConfig::default();

            assert_eq!(config.tap_timeout, Duration::from_millis(300));
            assert_eq!(config.double_tap_timeout, Duration::from_millis(300));
            assert_eq!(config.long_press_timeout, Duration::from_millis(500));
            assert_eq!(config.min_drag_distance, 10.0);
            assert_eq!(config.min_swipe_velocity, 500.0);
            assert_eq!(config.max_swipe_angle, 45.0_f32.to_radians());
            assert_eq!(config.pinch_slop, 20.0);
            assert_eq!(config.rotate_slop, 10.0_f32.to_radians());
        }

        #[test]
        fn test_custom_config() {
            let config = GestureConfig {
                tap_timeout: Duration::from_millis(200),
                double_tap_timeout: Duration::from_millis(400),
                long_press_timeout: Duration::from_millis(800),
                min_drag_distance: 5.0,
                min_swipe_velocity: 300.0,
                max_swipe_angle: 30.0_f32.to_radians(),
                pinch_slop: 15.0,
                rotate_slop: 5.0_f32.to_radians(),
            };

            assert_eq!(config.tap_timeout, Duration::from_millis(200));
            assert_eq!(config.long_press_timeout, Duration::from_millis(800));
            assert_eq!(config.min_drag_distance, 5.0);
            assert_eq!(config.min_swipe_velocity, 300.0);
            assert_eq!(config.pinch_slop, 15.0);
        }
    }

    mod test_tap_gesture_recognizer {
        use super::*;

        #[test]
        fn test_single_tap_recognition() {
            let config = GestureConfig::default();
            let mut recognizer = TapGestureRecognizer::new(config.clone());

            let events = vec![
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
                // Evento adicional para que expire el timeout y genere el tap
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(600), // Después del timeout
                },
            ];

            let mut recognized_events = vec![];
            for event in events {
                // First update state with add_pointer
                recognizer.add_pointer(event.clone());
                // Then get any generated events
                if let Some(gesture_event) = recognizer.handle_event(event) {
                    recognized_events.push(gesture_event);
                }
            }

            assert_eq!(recognized_events.len(), 1);
            match &recognized_events[0] {
                GestureEvent::Tap => {} // Correcto
                _ => panic!("Expected Tap event"),
            }
        }

        #[test]
        fn test_double_tap_recognition() {
            let config = GestureConfig::default();
            let mut recognizer = TapGestureRecognizer::new(config.clone());

            let events = vec![
                // Primer tap
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
                // Segundo tap
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(200),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(300),
                },
            ];

            let mut recognized_events = vec![];
            for event in events {
                // First update state with add_pointer
                recognizer.add_pointer(event.clone());
                // Then get any generated events
                if let Some(gesture_event) = recognizer.handle_event(event) {
                    recognized_events.push(gesture_event);
                }
            }

            assert_eq!(recognized_events.len(), 1);
            match &recognized_events[0] {
                GestureEvent::DoubleTap => {} // Correcto
                _ => panic!("Expected DoubleTap event"),
            }
        }

        #[test]
        fn test_tap_cancelled_by_movement() {
            let config = GestureConfig::default();
            let mut recognizer = TapGestureRecognizer::new(config.clone());

            let events = vec![
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 115.0, y: 100.0 }, // Movimiento > threshold
                    timestamp: Instant::now() + Duration::from_millis(50),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 115.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
            ];

            let mut recognized_events = vec![];
            for event in events {
                if let Some(gesture_event) = recognizer.handle_event(event) {
                    recognized_events.push(gesture_event);
                }
            }

            // No debería reconocer tap porque se movió
            assert!(recognized_events.is_empty());
        }
    }

    mod test_drag_gesture_recognizer {
        use super::*;

        #[test]
        fn test_drag_recognition() {
            let config = GestureConfig::default();
            let mut recognizer = DragGestureRecognizer::new(config.clone());

            let events = vec![
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 115.0, y: 100.0 }, // Movimiento > threshold
                    timestamp: Instant::now() + Duration::from_millis(50),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 130.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 130.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(150),
                },
            ];

            let mut recognized_events = vec![];
            for event in events {
                // First update state with add_pointer
                recognizer.add_pointer(event.clone());
                // Then get any generated events
                if let Some(gesture_event) = recognizer.handle_event(event) {
                    recognized_events.push(gesture_event);
                }
            }

            assert_eq!(recognized_events.len(), 3);
            match &recognized_events[0] {
                GestureEvent::DragStart(_) => {} // Correcto
                _ => panic!("Expected DragStart event"),
            }
            match &recognized_events[1] {
                GestureEvent::DragUpdate(_) => {} // Correcto
                _ => panic!("Expected DragUpdate event"),
            }
            match &recognized_events[2] {
                GestureEvent::DragEnd(_) => {} // Correcto
                _ => panic!("Expected DragEnd event"),
            }
        }
    }

    mod test_pinch_gesture_recognizer {
        use super::*;

        #[test]
        fn test_pinch_recognition() {
            let config = GestureConfig::default();
            let mut recognizer = PinchGestureRecognizer::new(config.clone());

            let events = vec![
                // Primer dedo
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                // Segundo dedo
                PointerEvent {
                    pointer_id: 2,
                    event_type: PointerEventType::Down,
                    position: Point { x: 120.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(10),
                },
                // Movimiento de pinch in
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 105.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(50),
                },
                PointerEvent {
                    pointer_id: 2,
                    event_type: PointerEventType::Move,
                    position: Point { x: 115.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(50),
                },
                // Liberar dedos
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 105.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
                PointerEvent {
                    pointer_id: 2,
                    event_type: PointerEventType::Up,
                    position: Point { x: 115.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
            ];

            let mut recognized_events = vec![];
            for event in events {
                // First update state with add_pointer
                recognizer.add_pointer(event.clone());
                // Then get any generated events
                if let Some(gesture_event) = recognizer.handle_event(event) {
                    recognized_events.push(gesture_event);
                }
            }

            assert!(!recognized_events.is_empty());
            // Verificar que se reconocieron eventos de pinch
            let has_pinch_start = recognized_events.iter().any(|e| matches!(e, GestureEvent::PinchStart(_)));
            assert!(has_pinch_start, "Should recognize pinch start");
        }
    }

    mod test_swipe_gesture_recognizer {
        use super::*;

        #[test]
        fn test_swipe_recognition() {
            let config = GestureConfig::default();
            let mut recognizer = SwipeGestureRecognizer::new(config.clone());

            let events = vec![
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 200.0, y: 100.0 }, // Movimiento horizontal rápido
                    timestamp: Instant::now() + Duration::from_millis(50),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 200.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
            ];

            let mut recognized_events = vec![];
            for event in events {
                // First update state with add_pointer
                recognizer.add_pointer(event.clone());
                // Then get any generated events
                if let Some(gesture_event) = recognizer.handle_event(event) {
                    recognized_events.push(gesture_event);
                }
            }

            assert!(!recognized_events.is_empty());
            // Verificar que se reconoció swipe
            let has_swipe = recognized_events.iter().any(|e| matches!(e, GestureEvent::Swipe(_)));
            assert!(has_swipe, "Should recognize swipe");
        }
    }

    mod test_gesture_arena {
        use super::*;

        #[test]
        fn test_gesture_competition() {
            let mut arena = GestureArena::new();

            // Simular dos gestos compitiendo
            let gesture1_id = arena.add(Box::new(TapGestureRecognizer::new(GestureConfig::default())));
            let gesture2_id = arena.add(Box::new(DragGestureRecognizer::new(GestureConfig::default())));

            // Uno gana la competencia
            arena.resolve(gesture1_id);

            // Verificar que hay un ganador
            assert!(arena.winner.is_some());
            assert_eq!(arena.winner.unwrap(), gesture1_id);
        }

        #[test]
        fn test_gesture_rejection() {
            let mut arena = GestureArena::new();

            let gesture_id = arena.add(Box::new(TapGestureRecognizer::new(GestureConfig::default())));

            // Rechazar el gesto
            if let Some(member) = arena.members.get_mut(&gesture_id) {
                member.recognizer.reject_gesture(0);
            }

            // Verificar que no hay ganador
            assert!(arena.winner.is_none());
        }
    }

    mod test_gesture_detector {
        use super::*;

        #[test]
        fn test_detector_initialization() {
            let widget = Box::new(BasicWidget::new());
            let detector = GestureDetector::new(widget);

            assert!(detector.arena.members.is_empty());
            assert!(detector.on_tap.is_none());
        }

        #[test]
        fn test_pointer_event_handling() {
            let widget = Box::new(BasicWidget::new());
            let mut detector = GestureDetector::new(widget);

            let event = PointerEvent {
                pointer_id: 1,
                event_type: PointerEventType::Down,
                position: Point { x: 100.0, y: 100.0 },
                timestamp: Instant::now(),
            };

            // El detector debería manejar el evento sin errores
            detector.handle_pointer_event(event);
        }

        #[test]
        fn test_default_gesture_setup() {
            let widget = Box::new(BasicWidget::new());
            let mut detector = GestureDetector::new(widget);

            detector.setup_default_gestures();

            // Verificar que tiene todos los reconocedores por defecto
            assert_eq!(detector.arena.members.len(), 5); // tap, drag, pinch, rotate, swipe
        }

        #[test]
        fn test_callback_registration() {
            let widget = Box::new(BasicWidget::new());
            let mut detector = GestureDetector::new(widget);

            let tap_called = Arc::new(Mutex::new(false));
            let tap_called_clone = tap_called.clone();

            let detector = detector.on_tap(move || {
                *tap_called_clone.lock().unwrap() = true;
            });

            assert!(detector.on_tap.is_some());
        }
    }

    mod test_gesture_composition {
        use super::*;

        #[test]
        fn test_multiple_gesture_recognition() {
            let widget = Box::new(BasicWidget::new());
            let mut detector = GestureDetector::new(widget);

            detector.setup_default_gestures();

            // Simular eventos que podrían activar múltiples gestos
            let events = vec![
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Down,
                    position: Point { x: 100.0, y: 100.0 },
                    timestamp: Instant::now(),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Move,
                    position: Point { x: 150.0, y: 100.0 }, // Movimiento para drag
                    timestamp: Instant::now() + Duration::from_millis(100),
                },
                PointerEvent {
                    pointer_id: 1,
                    event_type: PointerEventType::Up,
                    position: Point { x: 150.0, y: 100.0 },
                    timestamp: Instant::now() + Duration::from_millis(200),
                },
            ];

            let mut all_events = vec![];
            for event in events {
                let gesture_events = detector.arena.process_event(event);
                all_events.extend(gesture_events);
            }

            // Debería reconocer drag
            let has_drag = all_events.iter().any(|e| matches!(e, GestureEvent::DragStart(_) | GestureEvent::DragUpdate(_) | GestureEvent::DragEnd(_)));
            assert!(has_drag, "Should recognize drag gesture");
        }
    }
}