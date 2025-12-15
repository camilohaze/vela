//! UI Framework Module
//!
//! This module contains the core UI components and animation system for Vela.

pub mod animated;
pub mod curves;

pub use animated::{
    Animated, Curve, Tween, AnimatedBuilder, Widget,
    AnimationStatus, AnimationCallbacks, AdvancedAnimationController,
    Animation, AnimationSequence, AnimationParallel,
    AnimationControllerWrapper, AdvancedAnimationControllerWrapper
};
pub use curves::{EasingCurve, CubicBezier, interpolation};