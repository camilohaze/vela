//! Advanced Curves and Easing Functions for Vela UI Animations
//!
//! This module provides a comprehensive set of easing curves and interpolation
//! functions for smooth animations.

use std::f32::consts;

/// Cubic Bezier curve for custom easing
#[derive(Debug, Clone, Copy)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Evaluate the curve at parameter t (0.0 to 1.0)
    pub fn evaluate(&self, t: f32) -> f32 {
        // Use Newton-Raphson method to solve for y given x = t
        let mut t_guess = t;

        for _ in 0..5 {
            let x = self.sample_x(t_guess);
            let dx = self.sample_dx(t_guess);

            if dx != 0.0 {
                t_guess -= (x - t) / dx;
                t_guess = t_guess.clamp(0.0, 1.0);
            }
        }

        self.sample_y(t_guess)
    }

    fn sample_x(&self, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        3.0 * mt2 * t * self.x1 + 3.0 * mt * t2 * self.x2 + t3
    }

    fn sample_y(&self, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        3.0 * mt2 * t * self.y1 + 3.0 * mt * t2 * self.y2 + t3
    }

    fn sample_dx(&self, t: f32) -> f32 {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        3.0 * mt2 * self.x1 + 6.0 * mt * t * (self.x2 - self.x1) + 3.0 * t2 * (1.0 - self.x2)
    }
}

/// Comprehensive easing curve functions
#[derive(Debug, Clone, Copy)]
pub enum EasingCurve {
    // Basic curves
    Linear,

    // Sine curves
    SineIn,
    SineOut,
    SineInOut,

    // Quadratic curves
    QuadIn,
    QuadOut,
    QuadInOut,

    // Cubic curves
    CubicIn,
    CubicOut,
    CubicInOut,

    // Quartic curves
    QuartIn,
    QuartOut,
    QuartInOut,

    // Quintic curves
    QuintIn,
    QuintOut,
    QuintInOut,

    // Exponential curves
    ExpoIn,
    ExpoOut,
    ExpoInOut,

    // Circular curves
    CircIn,
    CircOut,
    CircInOut,

    // Back curves (overshoot)
    BackIn,
    BackOut,
    BackInOut,

    // Elastic curves
    ElasticIn,
    ElasticOut,
    ElasticInOut,

    // Bounce curves
    BounceIn,
    BounceOut,
    BounceInOut,

    // Custom cubic bezier
    CubicBezier(CubicBezier),
}

impl EasingCurve {
    /// Transform a linear progress (0.0 to 1.0) using the easing curve
    pub fn transform(&self, t: f32) -> f32 {
        match self {
            EasingCurve::Linear => t,

            // Sine
            EasingCurve::SineIn => 1.0 - ((t * consts::PI) / 2.0).cos(),
            EasingCurve::SineOut => ((t * consts::PI) / 2.0).sin(),
            EasingCurve::SineInOut => -(t * consts::PI).cos() / 2.0 + 0.5,

            // Quadratic
            EasingCurve::QuadIn => t * t,
            EasingCurve::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingCurve::QuadInOut => if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            },

            // Cubic
            EasingCurve::CubicIn => t * t * t,
            EasingCurve::CubicOut => 1.0 - (1.0 - t).powi(3),
            EasingCurve::CubicInOut => if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            },

            // Quartic
            EasingCurve::QuartIn => t * t * t * t,
            EasingCurve::QuartOut => 1.0 - (1.0 - t).powi(4),
            EasingCurve::QuartInOut => if t < 0.5 {
                8.0 * t * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
            },

            // Quintic
            EasingCurve::QuintIn => t * t * t * t * t,
            EasingCurve::QuintOut => 1.0 - (1.0 - t).powi(5),
            EasingCurve::QuintInOut => if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
            },

            // Exponential
            EasingCurve::ExpoIn => if t == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * t - 10.0) },
            EasingCurve::ExpoOut => if t == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * t) },
            EasingCurve::ExpoInOut => if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                2.0_f32.powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
            },

            // Circular
            EasingCurve::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            EasingCurve::CircOut => (1.0 - (1.0 - t).powi(2)).sqrt(),
            EasingCurve::CircInOut => if t < 0.5 {
                (1.0 - (1.0 - 2.0 * t).powi(2)).sqrt() / 2.0
            } else {
                ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
            },

            // Back (overshoot)
            EasingCurve::BackIn => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            },
            EasingCurve::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            },
            EasingCurve::BackInOut => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            },

            // Elastic
            EasingCurve::ElasticIn => if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c4 = (2.0 * consts::PI) / 3.0;
                -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
            },
            EasingCurve::ElasticOut => if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                let c4 = (2.0 * consts::PI) / 3.0;
                2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
            },
            EasingCurve::ElasticInOut => if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                let c5 = (2.0 * consts::PI) / 4.5;
                -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
            } else {
                let c5 = (2.0 * consts::PI) / 4.5;
                (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
            },

            // Bounce
            EasingCurve::BounceIn => 1.0 - Self::bounce_out(1.0 - t),
            EasingCurve::BounceOut => Self::bounce_out(t),
            EasingCurve::BounceInOut => if t < 0.5 {
                (1.0 - Self::bounce_out(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + Self::bounce_out(2.0 * t - 1.0)) / 2.0
            },

            // Custom cubic bezier
            EasingCurve::CubicBezier(bezier) => bezier.evaluate(t),
        }
    }

    fn bounce_out(t: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }
}

/// Interpolation functions for different data types
pub mod interpolation {
    use super::*;

    /// Linear interpolation between two values
    pub fn lerp<T>(a: T, b: T, t: f32) -> T
    where
        T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<f32, Output = T> + Copy,
    {
        a + (b - a) * t
    }

    /// Interpolate with easing curve
    pub fn lerp_eased<T>(a: T, b: T, t: f32, curve: EasingCurve) -> T
    where
        T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<f32, Output = T> + Copy,
    {
        let eased_t = curve.transform(t);
        lerp(a, b, eased_t)
    }

    /// Interpolate color (RGBA)
    pub fn lerp_color(start: [f32; 4], end: [f32; 4], t: f32) -> [f32; 4] {
        [
            lerp(start[0], end[0], t),
            lerp(start[1], end[1], t),
            lerp(start[2], end[2], t),
            lerp(start[3], end[3], t),
        ]
    }

    /// Interpolate 2D vector
    pub fn lerp_vec2(start: [f32; 2], end: [f32; 2], t: f32) -> [f32; 2] {
        [
            lerp(start[0], end[0], t),
            lerp(start[1], end[1], t),
        ]
    }

    /// Interpolate 3D vector
    pub fn lerp_vec3(start: [f32; 3], end: [f32; 3], t: f32) -> [f32; 3] {
        [
            lerp(start[0], end[0], t),
            lerp(start[1], end[1], t),
            lerp(start[2], end[2], t),
        ]
    }

    /// Smooth step interpolation (S-curve)
    pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Smoother step (higher order S-curve)
    pub fn smoother_step(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }
}

/// Predefined common easing curves
pub mod curves {
    use super::{EasingCurve, CubicBezier};

    pub const LINEAR: EasingCurve = EasingCurve::Linear;

    // Sine
    pub const SINE_IN: EasingCurve = EasingCurve::SineIn;
    pub const SINE_OUT: EasingCurve = EasingCurve::SineOut;
    pub const SINE_IN_OUT: EasingCurve = EasingCurve::SineInOut;

    // Cubic
    pub const CUBIC_IN: EasingCurve = EasingCurve::CubicIn;
    pub const CUBIC_OUT: EasingCurve = EasingCurve::CubicOut;
    pub const CUBIC_IN_OUT: EasingCurve = EasingCurve::CubicInOut;

    // Back
    pub const BACK_IN: EasingCurve = EasingCurve::BackIn;
    pub const BACK_OUT: EasingCurve = EasingCurve::BackOut;
    pub const BACK_IN_OUT: EasingCurve = EasingCurve::BackInOut;

    // Elastic
    pub const ELASTIC_IN: EasingCurve = EasingCurve::ElasticIn;
    pub const ELASTIC_OUT: EasingCurve = EasingCurve::ElasticOut;
    pub const ELASTIC_IN_OUT: EasingCurve = EasingCurve::ElasticInOut;

    // Bounce
    pub const BOUNCE_IN: EasingCurve = EasingCurve::BounceIn;
    pub const BOUNCE_OUT: EasingCurve = EasingCurve::BounceOut;
    pub const BOUNCE_IN_OUT: EasingCurve = EasingCurve::BounceInOut;

    // CSS standard curves
    pub fn ease() -> EasingCurve {
        EasingCurve::CubicBezier(CubicBezier::new(0.25, 0.1, 0.25, 1.0))
    }
    pub fn ease_in() -> EasingCurve {
        EasingCurve::CubicBezier(CubicBezier::new(0.42, 0.0, 1.0, 1.0))
    }
    pub fn ease_out() -> EasingCurve {
        EasingCurve::CubicBezier(CubicBezier::new(0.0, 0.0, 0.58, 1.0))
    }
    pub fn ease_in_out() -> EasingCurve {
        EasingCurve::CubicBezier(CubicBezier::new(0.42, 0.0, 0.58, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_bezier() {
        let bezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0); // CSS ease
        assert_eq!(bezier.evaluate(0.0), 0.0);
        assert_eq!(bezier.evaluate(1.0), 1.0);

        let mid = bezier.evaluate(0.5);
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_easing_curves_endpoints() {
        let curves = vec![
            EasingCurve::Linear,
            EasingCurve::SineIn,
            EasingCurve::CubicIn,
            EasingCurve::BackIn,
            EasingCurve::ElasticIn,
            EasingCurve::BounceIn,
        ];

        for curve in curves {
            assert_eq!(curve.transform(0.0), 0.0);
            assert_eq!(curve.transform(1.0), 1.0);
        }
    }

    #[test]
    fn test_easing_curves_monotonic() {
        let curve = EasingCurve::CubicIn;
        let mut prev = 0.0;

        for i in 1..=10 {
            let t = i as f32 / 10.0;
            let val = curve.transform(t);
            assert!(val >= prev);
            prev = val;
        }
    }

    #[test]
    fn test_interpolation_lerp() {
        assert_eq!(interpolation::lerp(0.0, 100.0, 0.0), 0.0);
        assert_eq!(interpolation::lerp(0.0, 100.0, 0.5), 50.0);
        assert_eq!(interpolation::lerp(0.0, 100.0, 1.0), 100.0);
    }

    #[test]
    fn test_interpolation_color() {
        let start = [0.0, 0.0, 0.0, 1.0];
        let end = [1.0, 1.0, 1.0, 0.0];
        let mid = interpolation::lerp_color(start, end, 0.5);

        assert_eq!(mid, [0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_smooth_step() {
        assert_eq!(interpolation::smooth_step(0.0, 1.0, 0.0), 0.0);
        assert_eq!(interpolation::smooth_step(0.0, 1.0, 1.0), 1.0);

        let mid = interpolation::smooth_step(0.0, 1.0, 0.5);
        assert!(mid > 0.5); // S-curve is above linear at midpoint
    }

    #[test]
    fn test_predefined_curves() {
        assert_eq!(curves::LINEAR.transform(0.5), 0.5);
        assert!(curves::ease().transform(0.5) > 0.5); // Ease curve is above linear
    }
}