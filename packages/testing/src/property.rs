/*!
# Property-Based Testing

Framework for property-based testing with random data generation.

## Example

```rust,no_run
use vela_testing::property::*;

// Property test
#[property_test]
async fn test_addition_commutative(a: i32, b: i32) {
    assert_eq!(add(a, b), add(b, a));
}

// Run property test
run_property_test!(test_addition_commutative, 1000);
```

*/

use rand::Rng;

/// Property test runner
pub struct PropertyTestRunner {
    iterations: usize,
}

impl PropertyTestRunner {
    pub fn new(iterations: usize) -> Self {
        Self { iterations }
    }

    /// Run property test with random data
    pub async fn run<F, Fut>(&self, test_fn: F) -> Result<(), String>
    where
        F: Fn(i32, i32) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut rng = rand::thread_rng();

        for i in 0..self.iterations {
            let a = rng.gen_range(-1000..1000);
            let b = rng.gen_range(-1000..1000);

            test_fn(a, b).await;
        }

        Ok(())
    }
}

/// Macro for property tests
#[macro_export]
macro_rules! property_test {
    ($(#[$attr:meta])* $vis:vis fn $name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)? $body:block) => {
        $(#[$attr])* $vis fn $name($($arg: $ty),*) $(-> $ret)? $body
    };
}

/// Macro to run property tests
#[macro_export]
macro_rules! run_property_test {
    ($test_fn:ident, $iterations:expr) => {
        {
            let runner = PropertyTestRunner::new($iterations);
            runner.run($test_fn).await.unwrap();
        }
    };
}

/// Arbitrary data generator
pub trait Arbitrary {
    fn arbitrary() -> Self;
}

impl Arbitrary for i32 {
    fn arbitrary() -> Self {
        rand::random()
    }
}

impl Arbitrary for String {
    fn arbitrary() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(0..100);
        (0..len).map(|_| rng.gen_range(b'a'..b'z') as char).collect()
    }
}