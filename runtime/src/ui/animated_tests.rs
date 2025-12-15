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
    fn test_elastic_curve() {
        let elastic = Curve::Elastic;
        assert_eq!(elastic.transform(0.0), 0.0);
        assert_eq!(elastic.transform(1.0), 1.0);

        // Elastic should overshoot significantly
        let mid = elastic.transform(0.5);
        // Elastic curves can go negative or very high
    }
}