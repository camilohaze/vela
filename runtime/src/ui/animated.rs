//! Animated Widget Implementation for Vela UI Framework
//!
//! This module provides smooth animations for UI components using interpolation
//! and reactive signals.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use crate::signals::{Signal, SignalValue};

/// Animation curve functions for different easing behaviors
#[derive(Debug, Clone, Copy)]
pub enum Curve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl Curve {
    /// Transform a linear progress (0.0 to 1.0) using the curve
    pub fn transform(&self, t: f32) -> f32 {
        match self {
            Curve::Linear => t,
            Curve::EaseIn => t * t,
            Curve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Curve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Curve::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            Curve::Elastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
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

/// Controls the playback of an animation
#[derive(Debug)]
pub struct AnimationController {
    duration: Duration,
    curve: Curve,
    start_time: Option<Instant>,
    is_playing: bool,
    is_completed: bool,
    progress_signal: Signal<f32>,
}

impl AnimationController {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            curve: Curve::Linear,
            start_time: None,
            is_playing: false,
            is_completed: false,
            progress_signal: Signal::new(0.0),
        }
    }

    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    pub fn forward(&mut self) {
        self.start_time = Some(Instant::now());
        self.is_playing = true;
        self.is_completed = false;
    }

    pub fn reverse(&mut self) {
        self.start_time = Some(Instant::now());
        self.is_playing = true;
        self.is_completed = false;
        // For reverse, we start from current progress and go backwards
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.is_playing = false;
        self.is_completed = false;
        self.progress_signal.set(0.0);
    }

    /// Update animation progress and return current value
    pub fn update(&mut self) -> f32 {
        if !self.is_playing || self.start_time.is_none() {
            return self.progress_signal.get();
        }

        let elapsed = self.start_time.unwrap().elapsed();
        let progress = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);

        if progress >= 1.0 {
            self.is_playing = false;
            self.is_completed = true;
            self.progress_signal.set(1.0);
            1.0
        } else {
            let curved_progress = self.curve.transform(progress);
            self.progress_signal.set(curved_progress);
            curved_progress
        }
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    pub fn progress(&self) -> f32 {
        self.progress_signal.get()
    }

    pub fn progress_signal(&self) -> &Signal<f32> {
        &self.progress_signal
    }
}

/// Animated widget that applies animations to its child properties
#[derive(Debug)]
pub struct Animated<T> {
    controller: AnimationController,
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
        controller: AnimationController,
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

    pub fn controller(&self) -> &AnimationController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut AnimationController {
        &mut self.controller
    }
}

/// Builder for creating animated widgets
pub struct AnimatedBuilder {
    controller: Option<AnimationController>,
    tweens: HashMap<String, Box<dyn std::any::Any>>,
}

impl AnimatedBuilder {
    pub fn new() -> Self {
        Self {
            controller: None,
            tweens: HashMap::new(),
        }
    }

    pub fn controller(mut self, controller: AnimationController) -> Self {
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
        let controller = self.controller.unwrap_or_else(|| AnimationController::new(Duration::from_secs(1)));
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
        let mut controller = AnimationController::new(Duration::from_millis(100));
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