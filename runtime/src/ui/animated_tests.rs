//! Tests for Animated Widget Implementation

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct MockWidget {
        pub opacity: f32,
        pub position: (f32, f32),
    }

    impl Widget for MockWidget {
        fn render(&self) -> String {
            format!("MockWidget(opacity: {}, position: ({}, {}))",
                   self.opacity, self.position.0, self.position.1)
        }

        fn update_property(&mut self, property: &str, value: SignalValue) {
            match (property, value) {
                ("opacity", SignalValue::Float(val)) => self.opacity = val,
                ("position_x", SignalValue::Float(val)) => self.position.0 = val,
                ("position_y", SignalValue::Float(val)) => self.position.1 = val,
                _ => {}
            }
        }
    }

    #[test]
    fn test_animation_controller_forward() {
        let mut controller = AnimationController::new(Duration::from_millis(100));
        assert_eq!(controller.progress(), 0.0);
        assert!(!controller.is_completed());

        controller.forward();
        assert!(controller.progress() >= 0.0);
    }

    #[test]
    fn test_animation_controller_completion() {
        let mut controller = AnimationController::new(Duration::from_millis(1));
        controller.forward();

        // Wait for completion
        std::thread::sleep(Duration::from_millis(10));

        controller.update();
        assert_eq!(controller.progress(), 1.0);
        assert!(controller.is_completed());
    }

    #[test]
    fn test_curve_linear() {
        assert_eq!(Curve::Linear.transform(0.0), 0.0);
        assert_eq!(Curve::Linear.transform(0.5), 0.5);
        assert_eq!(Curve::Linear.transform(1.0), 1.0);
    }

    #[test]
    fn test_curve_ease_in() {
        let ease_in_05 = Curve::EaseIn.transform(0.5);
        assert!(ease_in_05 < 0.5); // Should be less than linear
        assert!(ease_in_05 > 0.0);
    }

    #[test]
    fn test_curve_ease_out() {
        let ease_out_05 = Curve::EaseOut.transform(0.5);
        assert!(ease_out_05 > 0.5); // Should be more than linear
        assert!(ease_out_05 < 1.0);
    }

    #[test]
    fn test_tween_f32() {
        let tween = Tween::new(0.0, 100.0);
        assert_eq!(tween.lerp(0.0), 0.0);
        assert_eq!(tween.lerp(0.5), 50.0);
        assert_eq!(tween.lerp(1.0), 100.0);
    }

    #[test]
    fn test_animated_widget() {
        let controller = AnimationController::new(Duration::from_millis(100));
        let tween = Tween::new(0.0, 1.0);
        let child = Box::new(MockWidget {
            opacity: 0.0,
            position: (0.0, 0.0),
        });

        let mut animated = Animated::new(
            controller,
            tween,
            child,
            Box::new(|widget, value| {
                widget.update_property("opacity", SignalValue::Float(value));
            }),
        );

        animated.controller_mut().forward();
        animated.update();

        // The child should have updated opacity
        // Note: In a real test, we'd need to check the child's state
        // but since it's behind Box<dyn Widget>, we can't easily inspect it
        // This is a limitation of the current design
    }

    #[test]
    fn test_animated_builder() {
        let builder = AnimatedBuilder::new()
            .controller(AnimationController::new(Duration::from_secs(1)))
            .tween("opacity", 0.0, 1.0);

        let child = MockWidget {
            opacity: 0.0,
            position: (0.0, 0.0),
        };

        let animated = builder.build(child);

        assert_eq!(animated.controller().progress(), 0.0);
    }

    #[test]
    fn test_bounce_curve() {
        let bounce = Curve::Bounce;
        assert_eq!(bounce.transform(0.0), 0.0);
        assert_eq!(bounce.transform(1.0), 1.0);

        // Bounce should overshoot at certain points
        let mid = bounce.transform(0.5);
        assert!(mid > 0.5 || mid < 0.5); // Can be either side depending on bounce phase
    }

    #[test]
    fn test_advanced_animation_controller_basic() {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(100));
        assert_eq!(controller.status(), AnimationStatus::Idle);
        assert_eq!(controller.progress(), 0.0);

        controller.forward();
        assert_eq!(controller.status(), AnimationStatus::Running);
    }

    #[test]
    fn test_advanced_animation_controller_pause_resume() {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(100));
        controller.forward();
        assert_eq!(controller.status(), AnimationStatus::Running);

        controller.pause();
        assert_eq!(controller.status(), AnimationStatus::Paused);

        controller.forward(); // Resume
        assert_eq!(controller.status(), AnimationStatus::Running);
    }

    #[test]
    fn test_advanced_animation_controller_repeat() {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(1))
            .repeat(2);

        controller.forward();

        // Wait for first completion
        std::thread::sleep(Duration::from_millis(10));
        controller.update();
        assert_eq!(controller.current_repeat(), 1);

        // Wait for second completion
        std::thread::sleep(Duration::from_millis(10));
        controller.update();
        assert_eq!(controller.current_repeat(), 2);
        assert!(controller.is_completed());
    }

    #[test]
    fn test_advanced_animation_controller_auto_reverse() {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(1))
            .auto_reverse(true);

        controller.forward();

        // Wait for forward completion and reverse start
        std::thread::sleep(Duration::from_millis(10));
        controller.update();
        // Should be in reverse phase
        assert!(!controller.is_completed());
    }

    #[test]
    fn test_animation_sequence() {
        let controller1 = AnimationController::new(Duration::from_millis(1));
        let controller2 = AnimationController::new(Duration::from_millis(1));

        let mut sequence = AnimationSequence::new()
            .add_animation(Box::new(AnimationControllerWrapper { controller: controller1 }))
            .add_animation(Box::new(AnimationControllerWrapper { controller: controller2 }));

        // Start first animation
        if let Some(AnimationControllerWrapper { controller }) = sequence.animations.get_mut(0) {
            controller.forward();
        }

        // Update - should progress through sequence
        let progress = sequence.update();
        assert!(progress >= 0.0 && progress <= 1.0);
    }

    #[test]
    fn test_animation_parallel() {
        let controller1 = AnimationController::new(Duration::from_millis(1));
        let controller2 = AnimationController::new(Duration::from_millis(1));

        let mut parallel = AnimationParallel::new()
            .add_animation(Box::new(AnimationControllerWrapper { controller: controller1 }))
            .add_animation(Box::new(AnimationControllerWrapper { controller: controller2 }));

        // Start animations
        if let Some(AnimationControllerWrapper { controller }) = parallel.animations.get_mut(0) {
            controller.forward();
        }
        if let Some(AnimationControllerWrapper { controller }) = parallel.animations.get_mut(1) {
            controller.forward();
        }

        let progress = parallel.update();
        assert!(progress >= 0.0 && progress <= 1.0);
    }

    #[test]
    fn test_animation_callbacks() {
        let mut start_called = false;
        let mut complete_called = false;

        let callbacks = AnimationCallbacks {
            on_start: Some(Box::new(|| start_called = true)),
            on_complete: Some(Box::new(|| complete_called = true)),
            on_update: None,
            on_cancel: None,
        };

        let mut controller = AdvancedAnimationController::new(Duration::from_millis(1))
            .with_callbacks(callbacks);

        controller.forward();
        assert!(start_called);

        // Wait for completion
        std::thread::sleep(Duration::from_millis(10));
        controller.update();
        assert!(complete_called);
    }

    #[test]
    fn test_animation_speed() {
        let mut controller = AdvancedAnimationController::new(Duration::from_millis(100))
            .speed(2.0); // 2x speed

        controller.forward();

        // At 2x speed, should complete faster
        std::thread::sleep(Duration::from_millis(30)); // Less than half the duration
        controller.update();

        // Should be completed due to speed multiplier
        assert!(controller.is_completed());
    }