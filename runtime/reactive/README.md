# Vela Reactive System

A high-performance reactive programming library for Rust, providing the core primitives for building reactive applications in Vela.

## Features

- **Signal<T>**: Mutable reactive state with automatic dependency tracking
- **Computed<T>**: Lazy reactive derived values
- **Effect**: Side effects that run when dependencies change
- **Watch**: Reactive watchers for observing changes
- **Batch**: Group multiple updates for efficient processing
- **Scheduler**: Advanced update scheduling with prioritization

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vela-reactive = "0.1.0"
```

### Basic Usage

```rust
use vela_reactive::{signal, Signal, Computed, Effect, Watch, Batch};

// Create a reactive signal
let counter = signal(0);
println!("Counter: {}", counter.get()); // 0

// Create a computed value
let doubled = Computed::new(move || counter.get() * 2);
println!("Doubled: {}", doubled.get()); // 0

// Update the signal
counter.set(5);
println!("Counter: {}, Doubled: {}", counter.get(), doubled.get()); // 5, 10

// Create an effect that runs when dependencies change
let _effect = Effect::new(move || {
    println!("Counter changed to: {}", counter.get());
});

// Create a watcher
let _watch = Watch::new(counter.clone(), |value| {
    println!("Watch: counter is now {}", value);
});

// Batch multiple updates
let batch = Batch::new();
batch.batch(|| {
    counter.set(10);
    // Other updates...
});
```

## API Reference

### Signal<T>

```rust
// Create a signal
let signal = signal(42);

// Get value
let value = signal.get();

// Set value
signal.set(43);

// Subscribe to changes
let unsubscribe = signal.subscribe(|new_value| {
    println!("Value changed to: {}", new_value);
});

// Unsubscribe
unsubscribe();
```

### Computed<T>

```rust
// Create a computed value
let computed = Computed::new(|| {
    // Computation logic here
    42
});

// Get computed value (lazy evaluation)
let value = computed.get();
```

### Effect

```rust
// Create an effect
let effect = Effect::new(|| {
    // Effect logic here
    println!("Effect running!");
});

// Stop the effect
effect.stop();

// Resume the effect
effect.resume();
```

### Watch

```rust
// Create a watcher
let watch = Watch::new(signal, |value| {
    println!("Value changed: {}", value);
});

// Stop watching
watch.stop();

// Resume watching
watch.resume();
```

### Batch

```rust
// Create a batch context
let batch = Batch::new();

// Batch multiple updates
batch.batch(|| {
    signal1.set(1);
    signal2.set(2);
    // All updates processed together
});
```

## Architecture

The reactive system is built around several core concepts:

1. **Dependency Tracking**: Automatic tracking of dependencies between reactive values
2. **Lazy Evaluation**: Computed values are only recalculated when needed
3. **Update Batching**: Multiple updates can be grouped for efficiency
4. **Priority Scheduling**: Updates are processed in priority order
5. **Thread Safety**: All primitives are thread-safe using Arc and RwLock

## Performance

The reactive system is designed for high performance:

- Lazy evaluation prevents unnecessary computations
- Update coalescing reduces redundant work
- Priority-based scheduling ensures critical updates run first
- Thread-safe operations with minimal locking

## Examples

See the `examples/` directory for more detailed usage examples.

## Benchmarks

Run benchmarks with:

```bash
cargo bench
```

## Contributing

Contributions are welcome! Please see the main Vela repository for contribution guidelines.

## License

MIT OR Apache-2.0