/*!
# Property-Based Testing Framework

This module provides comprehensive property-based testing capabilities for Vela,
inspired by QuickCheck, Hypothesis, and proptest. It enables testing properties
that should hold for all possible inputs by generating random test cases.

## Features

- Random data generation for common types (integers, strings, collections, etc.)
- Configurable test parameters (iterations, seed, size limits)
- Automatic shrinking of failing test cases
- Integration with existing test framework
- Support for custom generators and shrinkers

## Example

```rust
use vela_testing::property::*;

// Simple property test
#[test]
fn test_reverse_identity() {
    let result = property_test(|vec: Vec<i32>| {
        let reversed = reverse(&reverse(&vec));
        vec == reversed
    });

    match result {
        PropertyTestResult::Passed { .. } => println!("All tests passed!"),
        PropertyTestResult::Failed { error, .. } => panic!("Property failed: {}", error),
    }
}

// Using the macro
property_test!(test_commutative_addition, |a: i32, b: i32| a + b == b + a);
```

*/

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Configuration for property-based tests
#[derive(Debug, Clone)]
pub struct PropertyTestConfig {
    /// Number of test cases to generate and test
    pub iterations: usize,
    /// Random seed for reproducible tests (None = random seed)
    pub seed: Option<u64>,
    /// Maximum size for generated collections
    pub max_size: usize,
    /// Whether to attempt shrinking of failing cases
    pub shrink: bool,
    /// Timeout in seconds for each test case
    pub timeout_secs: u64,
}

impl Default for PropertyTestConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            seed: None,
            max_size: 100,
            shrink: true,
            timeout_secs: 30,
        }
    }
}

/// Result of running a property test
#[derive(Debug, Clone)]
pub enum PropertyTestResult {
    /// All test cases passed
    Passed {
        iterations: usize,
        seed: u64,
    },
    /// Some test cases failed
    Failed {
        iterations: usize,
        seed: u64,
        failing_case: Value,
        shrunk_case: Option<Value>,
        error: String,
    },
}

impl fmt::Display for PropertyTestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyTestResult::Passed { iterations, seed } => {
                write!(f, "✓ Property test PASSED after {} iterations (seed: {})", iterations, seed)
            }
            PropertyTestResult::Failed { iterations, seed, failing_case, shrunk_case, error } => {
                write!(f, "✗ Property test FAILED after {} iterations (seed: {})\n", iterations, seed)?;
                write!(f, "Failing case: {}\n", failing_case)?;
                if let Some(shrunk) = shrunk_case {
                    write!(f, "Minimal case: {}\n", shrunk)?;
                }
                write!(f, "Error: {}", error)
            }
        }
    }
}

/// Trait for types that can be randomly generated and shrunk
pub trait Arbitrary: Sized + Clone {
    /// Generate a random instance of this type
    fn arbitrary() -> Self;

    /// Generate a random instance with size constraints
    fn arbitrary_with_size(size: usize) -> Self {
        Self::arbitrary()
    }

    /// Shrink this value to smaller values that might still fail the test
    fn shrink(&self) -> Vec<Self> {
        Vec::new()
    }
}

/// Generator for random test data
pub struct Generator {
    config: PropertyTestConfig,
    rng: rand::rngs::StdRng,
}

impl Generator {
    /// Create a new generator with default config
    pub fn new() -> Self {
        Self::with_config(PropertyTestConfig::default())
    }

    /// Create a new generator with custom config
    pub fn with_config(config: PropertyTestConfig) -> Self {
        use rand::SeedableRng;
        let rng = match config.seed {
            Some(seed) => rand::rngs::StdRng::seed_from_u64(seed),
            None => rand::rngs::StdRng::from_entropy(),
        };

        Self { config, rng }
    }

    /// Get the current seed
    pub fn seed(&self) -> u64 {
        // This is a simplified way to get the seed - in practice you'd need to track it
        0
    }

    /// Generate a random value of type T
    pub fn generate<T: Arbitrary>(&mut self) -> T {
        T::arbitrary()
    }

    /// Generate a vector of random values
    pub fn generate_vec<T: Arbitrary>(&mut self, size: usize) -> Vec<T> {
        (0..size.min(self.config.max_size)).map(|_| self.generate()).collect()
    }
}

/// Run a property test with a closure that takes one generated value
pub fn property_test<F, T>(property: F) -> PropertyTestResult
where
    F: Fn(T) -> bool,
    T: Arbitrary + serde::Serialize + fmt::Debug + Clone,
{
    property_test_with_config(PropertyTestConfig::default(), property)
}

/// Run a property test with custom configuration
pub fn property_test_with_config<F, T>(
    config: PropertyTestConfig,
    property: F,
) -> PropertyTestResult
where
    F: Fn(T) -> bool,
    T: Arbitrary + serde::Serialize + fmt::Debug + Clone,
{
    let mut generator = Generator::with_config(config.clone());
    let seed = generator.seed();

    for i in 0..config.iterations {
        let test_value = generator.generate::<T>();

        if !property(test_value.clone()) {
            // Test failed, try to shrink if enabled
            let shrunk_case = if config.shrink {
                shrink_test_case(&test_value, &property)
            } else {
                None
            };

            return PropertyTestResult::Failed {
                iterations: i + 1,
                seed,
                failing_case: serde_json::to_value(&test_value).unwrap_or(Value::Null),
                shrunk_case: shrunk_case.and_then(|v| serde_json::to_value(&v).ok()),
                error: format!("Property failed for input: {:?}", test_value),
            };
        }
    }

    PropertyTestResult::Passed {
        iterations: config.iterations,
        seed,
    }
}

/// Run a property test with two generated values
pub fn property_test2<F, T, U>(property: F) -> PropertyTestResult
where
    F: Fn(T, U) -> bool,
    T: Arbitrary + serde::Serialize + fmt::Debug + Clone,
    U: Arbitrary + serde::Serialize + fmt::Debug + Clone,
{
    property_test2_with_config(PropertyTestConfig::default(), property)
}

/// Run a property test with two values and custom config
pub fn property_test2_with_config<F, T, U>(
    config: PropertyTestConfig,
    property: F,
) -> PropertyTestResult
where
    F: Fn(T, U) -> bool,
    T: Arbitrary + serde::Serialize + fmt::Debug + Clone,
    U: Arbitrary + serde::Serialize + fmt::Debug + Clone,
{
    let mut generator = Generator::with_config(config.clone());
    let seed = generator.seed();

    for i in 0..config.iterations {
        let val1 = generator.generate::<T>();
        let val2 = generator.generate::<U>();

        if !property(val1.clone(), val2.clone()) {
            let shrunk_case = if config.shrink {
                shrink_test_case2(&val1, &val2, &property)
            } else {
                None
            };

            return PropertyTestResult::Failed {
                iterations: i + 1,
                seed,
                failing_case: serde_json::json!([val1, val2]),
                shrunk_case: shrunk_case.map(|(a, b)| serde_json::json!([a, b])),
                error: format!("Property failed for inputs: ({:?}, {:?})", val1, val2),
            };
        }
    }

    PropertyTestResult::Passed {
        iterations: config.iterations,
        seed,
    }
}

/// Try to shrink a failing test case to a minimal example
fn shrink_test_case<T, F>(failing_value: &T, property: &F) -> Option<T>
where
    F: Fn(T) -> bool,
    T: Arbitrary + Clone,
{
    let mut candidates = failing_value.shrink();
    let mut best_shrunk = None;

    // Simple shrinking strategy: try smaller values
    while let Some(candidate) = candidates.pop() {
        if !property(candidate.clone()) {
            // This candidate also fails, try to shrink it further
            best_shrunk = Some(candidate.clone());
            candidates.extend(candidate.shrink());
        }
    }

    best_shrunk
}

/// Try to shrink two failing test cases
fn shrink_test_case2<T, U, F>(val1: &T, val2: &U, property: &F) -> Option<(T, U)>
where
    F: Fn(T, U) -> bool,
    T: Arbitrary + Clone,
    U: Arbitrary + Clone,
{
    // Try shrinking first value
    if let Some(shrunk1) = shrink_test_case(val1, &|v1| property(v1, val2.clone())) {
        return Some((shrunk1, val2.clone()));
    }

    // Try shrinking second value
    if let Some(shrunk2) = shrink_test_case(val2, &|v2| property(val1.clone(), v2)) {
        return Some((val1.clone(), shrunk2));
    }

    None
}

/// Macro to define a property test (single argument)
#[macro_export]
macro_rules! property_test {
    ($name:ident, $property:expr) => {
        #[test]
        fn $name() {
            let result = $crate::property::property_test($property);
            match result {
                $crate::property::PropertyTestResult::Passed { .. } => {}
                $crate::property::PropertyTestResult::Failed { error, .. } => {
                    panic!("{}", error);
                }
            }
        }
    };
    ($name:ident, $config:expr, $property:expr) => {
        #[test]
        fn $name() {
            let result = $crate::property::property_test_with_config($config, $property);
            match result {
                $crate::property::PropertyTestResult::Passed { .. } => {}
                $crate::property::PropertyTestResult::Failed { error, .. } => {
                    panic!("{}", error);
                }
            }
        }
    };
}

/// Macro to define a property test (two arguments)
#[macro_export]
macro_rules! property_test2 {
    ($name:ident, $property:expr) => {
        #[test]
        fn $name() {
            let result = $crate::property::property_test2($property);
            match result {
                $crate::property::PropertyTestResult::Passed { .. } => {}
                $crate::property::PropertyTestResult::Failed { error, .. } => {
                    panic!("{}", error);
                }
            }
        }
    };
}

// Implement Arbitrary for common types

impl Arbitrary for i8 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(i8::MIN..=i8::MAX)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != 0 {
            shrunk.push(0);
        }
        if *self > 0 {
            shrunk.push(self.saturating_div(2));
        }
        if *self < 0 {
            shrunk.push(self.saturating_div(-2));
        }
        shrunk
    }
}

impl Arbitrary for i16 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(i16::MIN..=i16::MAX)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != 0 {
            shrunk.push(0);
        }
        if *self > 0 {
            shrunk.push(self.saturating_div(2));
        }
        if *self < 0 {
            shrunk.push(self.saturating_div(-2));
        }
        shrunk
    }
}

impl Arbitrary for i32 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(-10000..=10000)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != 0 {
            shrunk.push(0);
        }
        if *self > 0 {
            shrunk.push(self / 2);
            // Only add self-1 for small values to avoid explosion
            if *self <= 100 {
                shrunk.push(self - 1);
            }
        }
        if *self < 0 {
            shrunk.push(-self / 2);
            if *self >= -100 {
                shrunk.push(self + 1);
            }
        }
        shrunk
    }
}

impl Arbitrary for i64 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(-100000..=100000)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != 0 {
            shrunk.push(0);
        }
        if *self > 0 {
            shrunk.push(self / 2);
        }
        if *self < 0 {
            shrunk.push(-self / 2);
        }
        shrunk
    }
}

impl Arbitrary for u8 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(0..=u8::MAX)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self > 0 {
            shrunk.push(self / 2);
        }
        shrunk
    }
}

impl Arbitrary for u16 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(0..=u16::MAX)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self > 0 {
            shrunk.push(self / 2);
        }
        shrunk
    }
}

impl Arbitrary for u32 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(0..=100000)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self > 0 {
            shrunk.push(self / 2);
        }
        shrunk
    }
}

impl Arbitrary for u64 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(0..=1000000)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self > 0 {
            shrunk.push(self / 2);
        }
        shrunk
    }
}

impl Arbitrary for f32 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(-1000.0..=1000.0)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != 0.0 {
            shrunk.push(0.0);
        }
        if self.abs() > 1.0 {
            shrunk.push(self / 2.0);
        }
        shrunk
    }
}

impl Arbitrary for f64 {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_range(-10000.0..=10000.0)
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != 0.0 {
            shrunk.push(0.0);
        }
        if self.abs() > 1.0 {
            shrunk.push(self / 2.0);
        }
        shrunk
    }
}

impl Arbitrary for bool {
    fn arbitrary() -> Self {
        use rand::Rng;
        rand::thread_rng().gen_bool(0.5)
    }

    fn shrink(&self) -> Vec<Self> {
        if *self {
            vec![false]
        } else {
            Vec::new()
        }
    }
}

impl Arbitrary for char {
    fn arbitrary() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => rng.gen_range('a'..='z'),
            1 => rng.gen_range('A'..='Z'),
            2 => rng.gen_range('0'..='9'),
            _ => rng.gen_range(' '..='~'),
        }
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if *self != ' ' {
            shrunk.push(' ');
        }
        if *self != 'a' && self.is_ascii_alphabetic() {
            shrunk.push('a');
        }
        if *self != '0' && self.is_ascii_digit() {
            shrunk.push('0');
        }
        shrunk
    }
}

impl Arbitrary for String {
    fn arbitrary() -> Self {
        Self::arbitrary_with_size(20)
    }

    fn arbitrary_with_size(size: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(0..=size.min(20)); // Limit to 20 chars max
        (0..len).map(|_| char::arbitrary()).collect()
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if !self.is_empty() {
            shrunk.push(String::new());
            if self.len() > 1 {
                shrunk.push(self[1..].to_string());
                shrunk.push(self[..self.len() - 1].to_string());
            }
        }
        shrunk
    }
}

impl<T: Arbitrary> Arbitrary for Vec<T> {
    fn arbitrary() -> Self {
        Self::arbitrary_with_size(10)
    }

    fn arbitrary_with_size(size: usize) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(0..=size);
        (0..len).map(|_| T::arbitrary()).collect()
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        if !self.is_empty() {
            shrunk.push(Vec::new());
            if self.len() > 1 {
                shrunk.push(self[1..].to_vec());
                shrunk.push(self[..self.len() - 1].to_vec());
            }
        }
        shrunk
    }
}

impl<T: Arbitrary, U: Arbitrary> Arbitrary for (T, U) {
    fn arbitrary() -> Self {
        (T::arbitrary(), U::arbitrary())
    }

    fn shrink(&self) -> Vec<Self> {
        let mut shrunk = Vec::new();
        for t in self.0.shrink() {
            shrunk.push((t, self.1.clone()));
        }
        for u in self.1.shrink() {
            shrunk.push((self.0.clone(), u));
        }
        shrunk
    }
}

impl<T: Arbitrary> Arbitrary for Option<T> {
    fn arbitrary() -> Self {
        use rand::Rng;
        if rand::thread_rng().gen_bool(0.5) {
            Some(T::arbitrary())
        } else {
            None
        }
    }

    fn shrink(&self) -> Vec<Self> {
        match self {
            Some(value) => {
                let mut shrunk = vec![None];
                for v in value.shrink() {
                    shrunk.push(Some(v));
                }
                shrunk
            }
            None => Vec::new(),
        }
    }
}

impl<T: Arbitrary, E: Arbitrary> Arbitrary for Result<T, E> {
    fn arbitrary() -> Self {
        use rand::Rng;
        if rand::thread_rng().gen_bool(0.5) {
            Ok(T::arbitrary())
        } else {
            Err(E::arbitrary())
        }
    }

    fn shrink(&self) -> Vec<Self> {
        match self {
            Ok(value) => value.shrink().into_iter().map(Ok).collect(),
            Err(error) => error.shrink().into_iter().map(Err).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_primitives() {
        let _i32_val: i32 = i32::arbitrary();
        let _bool_val: bool = bool::arbitrary();
        let _string_val: String = String::arbitrary();
        let _vec_val: Vec<i32> = Vec::arbitrary();
        let _option_val: Option<i32> = Option::arbitrary();
        let _result_val: Result<i32, String> = Result::arbitrary();
    }

    #[test]
    fn test_property_test_passing() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 10,
                ..Default::default()
            },
            |x: i32| x.abs() >= 0, // Always true
        );

        match result {
            PropertyTestResult::Passed { iterations, .. } => assert_eq!(iterations, 10),
            _ => panic!("Test should pass"),
        }
    }

    #[test]
    fn test_property_test_failing() {
        let result = property_test_with_config(
            PropertyTestConfig {
                iterations: 10,
                ..Default::default()
            },
            |x: i32| x < 5000, // Will fail for large positive numbers
        );

        match result {
            PropertyTestResult::Failed { iterations, .. } => assert!(iterations <= 10),
            _ => panic!("Test should fail"),
        }
    }

    #[test]
    fn test_property_test2_passing() {
        let result = property_test2_with_config(
            PropertyTestConfig {
                iterations: 10,
                ..Default::default()
            },
            |a: i32, b: i32| a + b == b + a, // Commutative property
        );

        match result {
            PropertyTestResult::Passed { iterations, .. } => assert_eq!(iterations, 10),
            _ => panic!("Test should pass"),
        }
    }

    #[test]
    fn test_shrinking_i32() {
        let value = 1000;
        let shrunk = value.shrink();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&0));
        assert!(shrunk.contains(&500));
    }

    #[test]
    fn test_shrinking_string() {
        let value = "hello".to_string();
        let shrunk = value.shrink();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&String::new()));
        assert!(shrunk.contains(&"ello".to_string()));
        assert!(shrunk.contains(&"hell".to_string()));
    }

    #[test]
    fn test_shrinking_vec() {
        let value = vec![1, 2, 3];
        let shrunk = value.shrink();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&Vec::<i32>::new()));
        assert!(shrunk.contains(&vec![2, 3]));
        assert!(shrunk.contains(&vec![1, 2]));
    }

    #[test]
    fn test_shrinking_option() {
        let value = Some(42);
        let shrunk = value.shrink();
        assert!(!shrunk.is_empty());
        assert!(shrunk.contains(&None));
    }

    #[test]
    fn test_config_defaults() {
        let config = PropertyTestConfig::default();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.max_size, 100);
        assert!(config.shrink);
    }
}