# Pipe Decorators for Vela

The `@pipe` decorator is a **hybrid decorator** that adapts its behavior based on the context where it's used. It supports both frontend UI pipes (Angular-style) and backend HTTP pipes (NestJS-style).

## Context-Aware Behavior

### Frontend Context (UI Pipes)

When used with named parameters, `@pipe` creates UI transformation pipes for templates:

```rust
#[pipe(name = "currency", pure = true)]
pub struct CurrencyPipe;

impl CurrencyPipe {
    pub fn transform(&self, value: f64, currency: &str) -> String {
        format!("{} {:.2}", currency, value)
    }
}

// Usage in Vela templates
// ${amount | currency:'USD'}
```

### Backend Context (HTTP Pipes)

When used with pipe types, `@pipe` creates HTTP middleware for validation and transformation:

```rust
#[pipe(ValidationPipe, TransformPipe)]
pub struct UserPipe;

impl UserPipe {
    pub fn process(&self, data: serde_json::Value) -> Result<serde_json::Value, String> {
        // Validation and transformation logic
        Ok(data)
    }
}

// Usage in HTTP controllers
#[post("/users")]
pub fn create_user(#[body] user: User) {
    // Data is automatically validated and transformed
}
```

## Configuration Options

### Frontend Pipes
- `name`: String identifier for the pipe (required)
- `pure`: Boolean indicating if the pipe is pure/cacheable (default: true)

### Backend Pipes
- Pipe types: List of pipe classes to apply (ValidationPipe, TransformPipe, etc.)

## Built-in Pipe Types

### ValidationPipe
Performs input validation on HTTP requests.

### TransformPipe
Transforms data between different formats.

## Examples

### Currency Pipe (Frontend)
```rust
#[pipe(name = "currency", pure = true)]
pub struct CurrencyPipe;

impl CurrencyPipe {
    pub fn transform(&self, value: f64, currency: &str) -> String {
        format!("{} {:.2}", currency, value)
    }
}
```

### User Validation Pipe (Backend)
```rust
#[pipe(ValidationPipe, TransformPipe)]
pub struct UserPipe;

impl UserPipe {
    pub fn validate(&self, data: &serde_json::Value) -> Result<(), Vec<String>> {
        // Custom validation logic
        Ok(())
    }

    pub fn transform(&self, data: serde_json::Value) -> Result<serde_json::Value, String> {
        // Custom transformation logic
        Ok(data)
    }
}
```

## Auto-Detection

The decorator automatically detects the context based on the syntax:

- **Named parameters** (`name="..."`) → Frontend UI pipe
- **Type list** (`ValidationPipe, ...`) → Backend HTTP pipe

## Integration with Vela

This decorator integrates with Vela's compilation pipeline to generate appropriate code for each context:

- **Frontend**: Registers pipes for template usage
- **Backend**: Generates middleware for HTTP request processing

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```