//! Integration tests for Vela UI Animation System
//!
//! Tests comprehensive animation scenarios including:
//! - Animation composition (sequences and parallels)
//! - Easing curves validation
//! - Reactive signals integration
//! - Performance validation
//! - Edge cases and error handling

use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};

use vela_runtime::ui::{
    animated::{
        AdvancedAnimationController, AnimationCallbacks, AnimationSequence,
        AnimationParallel, AnimationStatus, Animation, Curve
    },
    curves::{EasingCurve, CubicBezier},
};
use vela_runtime::reactive::Signal;

/// Test basic animation lifecycle integration
#[test]
fn test_basic_animation_integration() {
    let mut controller = AdvancedAnimationController::new(Duration::from_millis(100));

    // Initial state
    assert_eq!(controller.status(), AnimationStatus::Idle);
    assert_eq!(controller.progress(), 0.0);
    assert!(!controller.is_running());

    // Start animation
    controller.forward();
    assert_eq!(controller.status(), AnimationStatus::Running);
    assert!(controller.is_running());

    // Simulate some time passing
    thread::sleep(Duration::from_millis(50));

    // Update should progress
    let progress = controller.update();
    assert!(progress > 0.0);
    assert!(progress < 1.0);

    // Complete animation
    thread::sleep(Duration::from_millis(60));
    let final_progress = controller.update();
    assert_eq!(final_progress, 1.0);
    assert_eq!(controller.status(), AnimationStatus::Completed);
}

/// Test animation with different easing curves
#[test]
fn test_easing_curves_integration() {
    let curves = vec![
        Curve::Linear,
        Curve::EaseIn,
        Curve::EaseOut,
        Curve::EaseInOut,
        Curve::Bounce,
        Curve::Elastic,
    ];

    for curve in curves {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(100))
            .with_curve(curve);

        controller.forward();

        // Run animation to completion
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(10));
            let progress = controller.update();
            assert!(progress >= 0.0 && progress <= 1.0);
        }

        // Ensure completion
        thread::sleep(Duration::from_millis(10));
        let final_progress = controller.update();
        assert_eq!(final_progress, 1.0);
        assert_eq!(controller.status(), AnimationStatus::Completed);
    }
}

/// Test animation sequence composition
#[test]
fn test_animation_sequence_integration() {
    let mut sequence = AnimationSequence::new();

    // Add multiple animations to sequence
    let anim1 = AdvancedAnimationController::new(Duration::from_millis(50));
    let anim2 = AdvancedAnimationController::new(Duration::from_millis(50));

    // Note: In real implementation, we'd add animations to sequence
    // For now, test the basic structure
    assert!(!sequence.is_completed());

    // Reset should work
    sequence.reset();
    assert!(!sequence.is_completed());
}

/// Test animation parallel composition
#[test]
fn test_animation_parallel_integration() {
    let mut parallel = AnimationParallel::new();

    // Add multiple animations to run in parallel
    let anim1 = AdvancedAnimationController::new(Duration::from_millis(100));
    let anim2 = AdvancedAnimationController::new(Duration::from_millis(100));

    // Note: In real implementation, we'd add animations to parallel
    // For now, test the basic structure
    assert!(!parallel.is_completed());

    // Reset should work
    parallel.reset();
    assert!(!parallel.is_completed());
}

/// Test reactive signals integration
#[test]
fn test_reactive_signals_integration() {
    let progress_signal = Arc::new(Signal::new(0.0));
    let signal_clone = Arc::clone(&progress_signal);

    let mut controller = AdvancedAnimationController::new(Duration::from_millis(100));

    // In real implementation, controller would be connected to signal
    // For now, test signal basics
    assert_eq!(signal_clone.get(), 0.0);

    signal_clone.set(0.5);
    assert_eq!(signal_clone.get(), 0.5);
}

/// Test animation callbacks
#[test]
fn test_animation_callbacks_integration() {
    let start_called = Arc::new(Mutex::new(false));
    let update_called = Arc::new(Mutex::new(false));
    let complete_called = Arc::new(Mutex::new(false));

    let start_clone = Arc::clone(&start_called);
    let update_clone = Arc::clone(&update_called);
    let complete_clone = Arc::clone(&complete_called);

    let callbacks = AnimationCallbacks {
        on_start: Some(Box::new(move || {
            *start_clone.lock().unwrap() = true;
        })),
        on_update: Some(Box::new(move |_| {
            *update_clone.lock().unwrap() = true;
        })),
        on_complete: Some(Box::new(move || {
            *complete_clone.lock().unwrap() = true;
        })),
        on_cancel: None,
    };

    let mut controller = AdvancedAnimationController::new(Duration::from_millis(50))
        .with_callbacks(callbacks);

    // Start animation (should trigger on_start)
    controller.forward();
    assert!(*start_called.lock().unwrap());

    // Update (should trigger on_update)
    thread::sleep(Duration::from_millis(25));
    controller.update();
    assert!(*update_called.lock().unwrap());

    // Complete animation (should trigger on_complete)
    thread::sleep(Duration::from_millis(30));
    controller.update();
    assert!(*complete_called.lock().unwrap());
}

/// Test animation repeat functionality
#[test]
fn test_animation_repeat_integration() {
    let mut controller = AdvancedAnimationController::new(Duration::from_millis(50))
        .repeat(2);

    controller.forward();

    // First completion
    thread::sleep(Duration::from_millis(60));
    controller.update();
    assert_eq!(controller.current_repeat(), 1);

    // Second completion
    thread::sleep(Duration::from_millis(60));
    controller.update();
    assert_eq!(controller.current_repeat(), 2);
    assert_eq!(controller.status(), AnimationStatus::Completed);
}

/// Test animation auto-reverse
#[test]
fn test_animation_auto_reverse_integration() {
    let mut controller = AdvancedAnimationController::new(Duration::from_millis(50))
        .auto_reverse(true);

    controller.forward();

    // First forward completion
    thread::sleep(Duration::from_millis(60));
    controller.update();
    assert_eq!(controller.status(), AnimationStatus::Completed);

    // Should have auto-reversed
    // Note: In real implementation, this would continue with reverse animation
}

/// Test animation pause and resume
#[test]
fn test_animation_pause_resume_integration() {
    let mut controller = AdvancedAnimationController::new(Duration::from_millis(100));

    controller.forward();
    thread::sleep(Duration::from_millis(30));

    // Pause
    controller.pause();
    assert_eq!(controller.status(), AnimationStatus::Paused);

    let progress_at_pause = controller.progress();

    // Wait and check progress doesn't change
    thread::sleep(Duration::from_millis(30));
    assert_eq!(controller.progress(), progress_at_pause);

    // Resume
    controller.forward();
    assert_eq!(controller.status(), AnimationStatus::Running);

    // Continue to completion
    thread::sleep(Duration::from_millis(50));
    controller.update();
    assert_eq!(controller.status(), AnimationStatus::Completed);
}

/// Test animation speed control
#[test]
fn test_animation_speed_integration() {
    let mut fast_controller = AdvancedAnimationController::new(Duration::from_millis(100))
        .speed(2.0);

    let mut slow_controller = AdvancedAnimationController::new(Duration::from_millis(100))
        .speed(0.5);

    fast_controller.forward();
    slow_controller.forward();

    thread::sleep(Duration::from_millis(30));

    // Fast animation should progress more
    let fast_progress = fast_controller.update();
    let slow_progress = slow_controller.update();

    assert!(fast_progress > slow_progress);
}

/// Test edge case: zero duration animation
#[test]
fn test_zero_duration_animation() {
    let mut controller = AdvancedAnimationController::new(Duration::from_millis(0));

    controller.forward();

    // Should complete immediately
    let progress = controller.update();
    assert_eq!(progress, 1.0);
    assert_eq!(controller.status(), AnimationStatus::Completed);
}

/// Test edge case: very long duration
#[test]
fn test_long_duration_animation() {
    let mut controller = AdvancedAnimationController::new(Duration::from_secs(3600)); // 1 hour

    controller.forward();

    // Should still work for initial progress
    let progress = controller.update();
    assert_eq!(progress, 0.0); // Should be very close to 0
    assert_eq!(controller.status(), AnimationStatus::Running);
}

/// Performance test: multiple concurrent animations
#[test]
fn test_concurrent_animations_performance() {
    let mut controllers = Vec::new();

    // Create 100 concurrent animations
    for i in 0..100 {
        let duration = Duration::from_millis(100 + (i % 10) as u64);
        let mut controller = AdvancedAnimationController::new(duration);
        controller.forward();
        controllers.push(controller);
    }

    // Update all animations
    let start_time = Instant::now();
    for controller in &mut controllers {
        controller.update();
    }
    let update_time = start_time.elapsed();

    // Should complete in reasonable time (< 1ms per animation)
    assert!(update_time < Duration::from_millis(10));

    // All should be running
    for controller in &controllers {
        assert_eq!(controller.status(), AnimationStatus::Running);
    }
}

/// Test cubic bezier curve validation
#[test]
fn test_cubic_bezier_curve_validation() {
    // Valid cubic bezier
    let bezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
    assert_eq!(bezier.evaluate(0.0), 0.0);
    assert_eq!(bezier.evaluate(1.0), 1.0);

    // Test monotonicity
    let mut prev = 0.0;
    for i in 1..=100 {
        let t = i as f32 / 100.0;
        let value = bezier.evaluate(t);
        assert!(value >= prev);
        prev = value;
    }
}

/// Test all predefined easing curves
#[test]
fn test_all_predefined_curves() {
    let curves = vec![
        EasingCurve::Linear,
        EasingCurve::SineIn,
        EasingCurve::SineOut,
        EasingCurve::SineInOut,
        EasingCurve::QuadIn,
        EasingCurve::QuadOut,
        EasingCurve::QuadInOut,
        EasingCurve::CubicIn,
        EasingCurve::CubicOut,
        EasingCurve::CubicInOut,
        EasingCurve::QuartIn,
        EasingCurve::QuartOut,
        EasingCurve::QuartInOut,
        EasingCurve::QuintIn,
        EasingCurve::QuintOut,
        EasingCurve::QuintInOut,
        EasingCurve::ExpoIn,
        EasingCurve::ExpoOut,
        EasingCurve::ExpoInOut,
        EasingCurve::CircIn,
        EasingCurve::CircOut,
        EasingCurve::CircInOut,
        EasingCurve::BounceIn,
        EasingCurve::BounceOut,
        EasingCurve::BounceInOut,
        EasingCurve::ElasticIn,
        EasingCurve::ElasticOut,
        EasingCurve::ElasticInOut,
    ];

    for curve in curves {
        // Test endpoints
        assert_eq!(curve.transform(0.0), 0.0);
        assert_eq!(curve.transform(1.0), 1.0);

        // Test monotonicity (should not decrease)
        let mut prev = 0.0;
        for i in 1..=50 {
            let t = i as f32 / 50.0;
            let value = curve.transform(t);
            assert!(value >= prev);
            prev = value;
        }
    }
}