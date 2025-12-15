//! UI Framework Module
//!
//! This module contains the core UI components, animation system, and gesture recognition for Vela.

pub mod animated;
pub mod curves;
pub mod gestures;

pub use animated::{
    Animated, Curve, Tween, AnimatedBuilder, Widget,
    AnimationStatus, AnimationCallbacks, AdvancedAnimationController,
    Animation, AnimationSequence, AnimationParallel,
    AnimationControllerWrapper, AdvancedAnimationControllerWrapper
};
pub use curves::{EasingCurve, CubicBezier, interpolation};
pub use gestures::{
    GestureDetector, GestureEvent, GestureConfig, GestureArena,
    Point, Velocity, PointerEvent, PointerEventType,
    DragStartDetails, DragUpdateDetails, DragEndDetails,
    PinchStartDetails, PinchUpdateDetails, PinchEndDetails,
    RotateStartDetails, RotateUpdateDetails, RotateEndDetails,
    SwipeDetails, SwipeDirection,
    GestureRecognizer, TapGestureRecognizer, DragGestureRecognizer,
    PinchGestureRecognizer, RotateGestureRecognizer, SwipeGestureRecognizer,
};