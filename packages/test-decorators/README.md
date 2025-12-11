# Test Decorators for Vela

This package provides procedural macros for test-related decorators in the Vela programming language.

## Decorators

### @test - Unit Tests

Marks functions as unit tests that can be executed by the Vela test runner.

```rust
#[test]
fn my_test() {
    assert_eq!(2 + 2, 4);
}

// With configuration
#[test(name = "custom test name", timeout = 1000, ignore = false)]
fn configured_test() {
    // Test with custom name and 1 second timeout
}

#[test(should_panic = "expected panic message")]
fn test_that_should_panic() {
    panic!("expected panic message");
}
```

**Configuration Options:**
- `name`: Custom test name (defaults to function name)
- `ignore`: Skip this test
- `should_panic`: Expected panic message for panic tests
- `timeout`: Maximum execution time in milliseconds

### @describe - BDD Test Groups

Groups related tests in BDD (Behavior-Driven Development) style.

```rust
#[describe("User authentication")]
mod auth_tests {
    #[it("should login with valid credentials")]
    fn test_valid_login() {
        // Test implementation
    }

    #[it("should reject invalid credentials")]
    fn test_invalid_login() {
        // Test implementation
    }
}
```

**Configuration Options:**
- `skip`: Skip the entire test suite

### @it - BDD Test Cases

Defines individual test cases within a BDD test suite.

```rust
#[it("should perform specific behavior")]
fn test_specific_behavior() {
    // Test implementation
}

// With configuration
#[it("should handle edge case", timeout = 2000, skip = false)]
fn test_edge_case() {
    // Test with 2 second timeout
}
```

**Configuration Options:**
- `timeout`: Maximum execution time in milliseconds
- `skip`: Skip this specific test

### @context - Alias for @describe

Provides better readability in some contexts.

```rust
#[context("Database operations")]
mod db_tests {
    // Same as @describe
}
```

### Lifecycle Hooks

#### @beforeEach - Setup Before Each Test

```rust
#[before_each]
fn setup() {
    // Runs before each test in the module
}

// With timeout
#[before_each(timeout = 500)]
fn setup_with_timeout() {
    // Setup with 500ms timeout
}
```

#### @afterEach - Cleanup After Each Test

```rust
#[after_each]
fn cleanup() {
    // Runs after each test in the module
}
```

#### @beforeAll - Global Setup

```rust
#[before_all]
fn global_setup() {
    // Runs once before all tests in the module
}
```

#### @afterAll - Global Cleanup

```rust
#[after_all]
fn global_cleanup() {
    // Runs once after all tests in the module
}
```

## Usage in Vela

These decorators are designed to work with Vela's testing framework. When you write tests in Vela code, the compiler will use these procedural macros to generate appropriate Rust test code.

## Examples

See `examples/usage.rs` for comprehensive examples of all decorators.

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Integration with Vela Compiler

This package is part of Vela's compiler toolchain. The decorators are processed during compilation to generate test executables that can be run with standard Rust testing tools.