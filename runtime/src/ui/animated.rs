//! Animated Widget Implementation for Vela UI Framework
//!
//! This module provides smooth animations for UI components using interpolation
//! and reactive signals.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use reactive::{Signal};

/// Value types that can be passed to widget properties
#[derive(Debug, Clone, PartialEq)]
pub enum SignalValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
}

use crate::ui::curves::EasingCurve;

/// Animation curve functions for different easing behaviors
#[derive(Debug, Clone, Copy)]
pub enum Curve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    // Legacy support - these map to EasingCurve
}

impl Curve {
    /// Transform a linear progress (0.0 to 1.0) using the curve
    pub fn transform(&self, t: f32) -> f32 {
        match self {
            Curve::Linear => EasingCurve::Linear.transform(t),
            Curve::EaseIn => EasingCurve::CubicIn.transform(t),
            Curve::EaseOut => EasingCurve::CubicOut.transform(t),
            Curve::EaseInOut => EasingCurve::CubicInOut.transform(t),
            Curve::Bounce => EasingCurve::BounceOut.transform(t),
            Curve::Elastic => EasingCurve::ElasticOut.transform(t),
        }
    }
}

/// Defines a range of values to interpolate between
#[derive(Debug, Clone)]
pub struct Tween<T> {
    pub begin: T,
    pub end: T,
}

impl<T> Tween<T> {
    pub fn new(begin: T, end: T) -> Self {
        Self { begin, end }
    }

    /// Interpolate between begin and end using progress t (0.0 to 1.0)
    pub fn lerp(&self, t: f32) -> T
    where
        T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<f32, Output = T> + Copy,
    {
        self.begin + (self.end - self.begin) * t
    }
}

/// Status of an animation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationStatus {
    Idle,
    Running,
    Paused,
    Completed,
    Cancelled,
}

/// Callbacks for animation events
#[derive(Default)]
pub struct AnimationCallbacks {
    pub on_start: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_update: Option<Box<dyn Fn(f32) + Send + Sync>>,
    pub on_complete: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_cancel: Option<Box<dyn Fn() + Send + Sync>>,
}

/// Advanced Animation Controller with full control features
pub struct AdvancedAnimationController {
    duration: Duration,
    curve: Curve,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    status: AnimationStatus,
    progress_signal: Signal<f32>,
    callbacks: AnimationCallbacks,
    repeat_count: Option<u32>,
    current_repeat: u32,
    auto_reverse: bool,
    is_reversing: bool,
    speed: f32,
}

impl AdvancedAnimationController {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            curve: Curve::Linear,
            start_time: None,
            pause_time: None,
            status: AnimationStatus::Idle,
            progress_signal: Signal::new(0.0),
            callbacks: AnimationCallbacks::default(),
            repeat_count: None,
            current_repeat: 0,
            auto_reverse: false,
            is_reversing: false,
            speed: 1.0,
        }
    }

    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    pub fn with_callbacks(mut self, callbacks: AnimationCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn repeat(mut self, count: u32) -> Self {
        self.repeat_count = Some(count);
        self
    }

    pub fn auto_reverse(mut self, auto_reverse: bool) -> Self {
        self.auto_reverse = auto_reverse;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed.max(0.1); // Minimum speed to avoid division by zero
        self
    }

    pub fn forward(&mut self) {
        if self.status == AnimationStatus::Idle || self.status == AnimationStatus::Completed {
            self.start_time = Some(Instant::now());
            self.pause_time = None;
            self.status = AnimationStatus::Running;
            self.current_repeat = 0;
            self.is_reversing = false;
            if let Some(ref callback) = self.callbacks.on_start {
                callback();
            }
        } else if self.status == AnimationStatus::Paused {
            // Resume from pause
            if let Some(pause_time) = self.pause_time {
                let paused_duration = Instant::now().duration_since(pause_time);
                self.start_time = Some(self.start_time.unwrap() + paused_duration);
            }
            self.pause_time = None;
            self.status = AnimationStatus::Running;
        }
    }

    pub fn reverse(&mut self) {
        if self.status == AnimationStatus::Idle || self.status == AnimationStatus::Completed {
            self.start_time = Some(Instant::now());
            self.pause_time = None;
            self.status = AnimationStatus::Running;
            self.is_reversing = true;
            if let Some(ref callback) = self.callbacks.on_start {
                callback();
            }
        }
    }

    pub fn pause(&mut self) {
        if self.status == AnimationStatus::Running {
            self.pause_time = Some(Instant::now());
            self.status = AnimationStatus::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.status = AnimationStatus::Idle;
        self.start_time = None;
        self.pause_time = None;
        self.progress_signal.set(0.0);
        self.current_repeat = 0;
        self.is_reversing = false;
        if let Some(ref callback) = self.callbacks.on_cancel {
            callback();
        }
    }

    pub fn reset(&mut self) {
        self.stop();
        self.status = AnimationStatus::Idle;
    }

    pub fn cancel(&mut self) {
        self.status = AnimationStatus::Cancelled;
        if let Some(ref callback) = self.callbacks.on_cancel {
            callback();
        }
    }

    /// Update animation progress and return current value
    pub fn update(&mut self) -> f32 {
        if self.status != AnimationStatus::Running || self.start_time.is_none() {
            return self.progress_signal.get();
        }

        let elapsed = self.start_time.unwrap().elapsed();
        let adjusted_duration = self.duration.div_f32(self.speed);
        let raw_progress = (elapsed.as_secs_f32() / adjusted_duration.as_secs_f32()).min(1.0);

        let progress = if self.is_reversing {
            1.0 - raw_progress
        } else {
            raw_progress
        };

        let curved_progress = self.curve.transform(progress);
        self.progress_signal.set(curved_progress);

        if let Some(ref callback) = self.callbacks.on_update {
            callback(curved_progress);
        }

        if progress >= 1.0 {
            self.handle_completion();
        }

        curved_progress
    }

    fn handle_completion(&mut self) {
        if self.auto_reverse && !self.is_reversing {
            // Start reverse animation
            self.is_reversing = true;
            self.start_time = Some(Instant::now());
        } else if let Some(max_repeats) = self.repeat_count {
            if self.current_repeat < max_repeats {
                // Start next repeat
                self.current_repeat += 1;
                self.start_time = Some(Instant::now());
                self.is_reversing = false;
            } else {
                // Animation fully completed
                self.status = AnimationStatus::Completed;
                self.progress_signal.set(if self.is_reversing { 0.0 } else { 1.0 });
                if let Some(ref callback) = self.callbacks.on_complete {
                    callback();
                }
            }
        } else {
            // Single animation completed
            self.status = AnimationStatus::Completed;
            self.progress_signal.set(if self.is_reversing { 0.0 } else { 1.0 });
            if let Some(ref callback) = self.callbacks.on_complete {
                callback();
            }
        }
    }

    pub fn status(&self) -> AnimationStatus {
        self.status
    }

    pub fn progress(&self) -> f32 {
        self.progress_signal.get()
    }

    pub fn progress_signal(&self) -> &Signal<f32> {
        &self.progress_signal
    }

    pub fn is_completed(&self) -> bool {
        self.status == AnimationStatus::Completed
    }

    pub fn is_running(&self) -> bool {
        self.status == AnimationStatus::Running
    }

    pub fn is_paused(&self) -> bool {
        self.status == AnimationStatus::Paused
    }

    pub fn current_repeat(&self) -> u32 {
        self.current_repeat
    }
}

/// Animation that can be sequenced or run in parallel
pub trait Animation {
    fn update(&mut self) -> f32;
    fn is_completed(&self) -> bool;
    fn reset(&mut self);
}

/// Sequence of animations that play one after another
pub struct AnimationSequence {
    animations: Vec<Box<dyn Animation>>,
    current_index: usize,
    is_completed: bool,
}

impl AnimationSequence {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            current_index: 0,
            is_completed: false,
        }
    }

    pub fn add_animation(mut self, animation: Box<dyn Animation>) -> Self {
        self.animations.push(animation);
        self
    }
}

impl Animation for AnimationSequence {
    fn update(&mut self) -> f32 {
        if self.is_completed || self.animations.is_empty() {
            return 1.0;
        }

        let current_anim = &mut self.animations[self.current_index];
        let progress = current_anim.update();

        if current_anim.is_completed() {
            self.current_index += 1;
            if self.current_index >= self.animations.len() {
                self.is_completed = true;
                return 1.0;
            }
        }

        // Return overall progress
        (self.current_index as f32 + progress) / self.animations.len() as f32
    }

    fn is_completed(&self) -> bool {
        self.is_completed
    }

    fn reset(&mut self) {
        self.current_index = 0;
        self.is_completed = false;
        for anim in &mut self.animations {
            anim.reset();
        }
    }
}

/// Animations that run in parallel
pub struct AnimationParallel {
    animations: Vec<Box<dyn Animation>>,
    is_completed: bool,
}

impl AnimationParallel {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            is_completed: false,
        }
    }

    pub fn add_animation(mut self, animation: Box<dyn Animation>) -> Self {
        self.animations.push(animation);
        self
    }
}

impl Animation for AnimationParallel {
    fn update(&mut self) -> f32 {
        if self.is_completed || self.animations.is_empty() {
            return 1.0;
        }

        let mut total_progress = 0.0;
        let mut all_completed = true;

        for anim in &mut self.animations {
            let progress = anim.update();
            total_progress += progress;
            if !anim.is_completed() {
                all_completed = false;
            }
        }

        if all_completed {
            self.is_completed = true;
            return 1.0;
        }

        total_progress / self.animations.len() as f32
    }

    fn is_completed(&self) -> bool {
        self.is_completed
    }

    fn reset(&mut self) {
        self.is_completed = false;
        for anim in &mut self.animations {
            anim.reset();
        }
    }
}

// Wrapper to make AnimationController implement Animation trait
pub struct AnimationControllerWrapper {
    pub controller: AdvancedAnimationController,
}

impl Animation for AnimationControllerWrapper {
    fn update(&mut self) -> f32 {
        self.controller.update()
    }

    fn is_completed(&self) -> bool {
        self.controller.is_completed()
    }

    fn reset(&mut self) {
        self.controller.reset()
    }
}

// Wrapper for AdvancedAnimationController
pub struct AdvancedAnimationControllerWrapper {
    pub controller: AdvancedAnimationController,
}

impl Animation for AdvancedAnimationControllerWrapper {
    fn update(&mut self) -> f32 {
        self.controller.update()
    }

    fn is_completed(&self) -> bool {
        self.controller.is_completed()
    }

    fn reset(&mut self) {
        self.controller.reset()
    }
}

/// Animated widget that applies animations to its child properties
pub struct Animated<T> {
    controller: AdvancedAnimationController,
    tween: Tween<T>,
    child: Box<dyn Widget>,
    property_updater: Box<dyn Fn(&mut dyn Widget, T)>,
}

pub trait Widget {
    fn render(&self) -> String;
    fn update_property(&mut self, property: &str, value: SignalValue);
}

impl<T> Animated<T>
where
    T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<f32, Output = T> + Copy + 'static,
{
    pub fn new(
        controller: AdvancedAnimationController,
        tween: Tween<T>,
        child: Box<dyn Widget>,
        property_updater: Box<dyn Fn(&mut dyn Widget, T)>,
    ) -> Self {
        Self {
            controller,
            tween,
            child,
            property_updater,
        }
    }

    pub fn update(&mut self) {
        let progress = self.controller.update();
        let value = self.tween.lerp(progress);
        (self.property_updater)(self.child.as_mut(), value);
    }

    pub fn render(&self) -> String {
        self.child.render()
    }

    pub fn controller(&self) -> &AdvancedAnimationController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut AdvancedAnimationController {
        &mut self.controller
    }
}

/// Builder for creating animated widgets
pub struct AnimatedBuilder {
    controller: Option<AdvancedAnimationController>,
    tweens: HashMap<String, Box<dyn std::any::Any>>,
}

impl AnimatedBuilder {
    pub fn new() -> Self {
        Self {
            controller: None,
            tweens: HashMap::new(),
        }
    }

    pub fn controller(mut self, controller: AdvancedAnimationController) -> Self {
        self.controller = Some(controller);
        self
    }

    pub fn tween<T: 'static>(mut self, property: &str, begin: T, end: T) -> Self {
        self.tweens.insert(property.to_string(), Box::new(Tween::new(begin, end)));
        self
    }

    pub fn build<W: Widget + 'static>(self, child: W) -> Animated<f32> {
        // For simplicity, create a basic animation
        // In a real implementation, this would handle multiple properties
        let controller = self.controller.unwrap_or_else(|| AdvancedAnimationController::new(Duration::from_secs(1)));
        let tween = Tween::new(0.0, 1.0);

        Animated::new(
            controller,
            tween,
            Box::new(child),
            Box::new(|widget, value| {
                // Generic property update - in practice, this would be more specific
                widget.update_property("opacity", SignalValue::Float(value));
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        opacity: f32,
    }

    impl Widget for TestWidget {
        fn render(&self) -> String {
            format!("TestWidget(opacity: {})", self.opacity)
        }

        fn update_property(&mut self, property: &str, value: SignalValue) {
            if property == "opacity" {
                if let SignalValue::Float(val) = value {
                    self.opacity = val;
                }
            }
        }
    }

    #[test]
    fn test_animation_controller() {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(100));
        controller.forward();

        // Initially should be 0
        assert_eq!(controller.progress(), 0.0);

        // After update, should progress
        std::thread::sleep(Duration::from_millis(50));
        controller.update();
        assert!(controller.progress() > 0.0);
    }

    #[test]
    fn test_curve_transform() {
        assert_eq!(Curve::Linear.transform(0.5), 0.5);
        assert!(Curve::EaseIn.transform(0.5) < 0.5);
        assert!(Curve::EaseOut.transform(0.5) > 0.5);
    }

    #[test]
    fn test_tween_lerp() {
        let tween = Tween::new(0.0, 100.0);
        assert_eq!(tween.lerp(0.0), 0.0);
        assert_eq!(tween.lerp(0.5), 50.0);
        assert_eq!(tween.lerp(1.0), 100.0);
    }
}