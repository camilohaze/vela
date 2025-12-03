//! # Basic Reactive Example
//!
//! This example demonstrates the basic usage of the Vela reactive system.

use vela_reactive::{signal, Signal, Computed, Effect, Watch, Batch};

fn main() {
    println!("🚀 Vela Reactive System Demo");
    println!("============================");

    // 1. Basic Signal
    println!("\n1. Basic Signal:");
    let counter = signal(0);
    println!("Initial value: {}", counter.get());
    counter.set(42);
    println!("After set(42): {}", counter.get());

    // 2. Computed Values
    println!("\n2. Computed Values:");
    let doubled = Computed::new(move || counter.get() * 2);
    println!("counter: {}, doubled: {}", counter.get(), doubled.get());

    counter.set(10);
    println!("After counter.set(10):");
    println!("counter: {}, doubled: {}", counter.get(), doubled.get());

    // 3. Effects
    println!("\n3. Effects:");
    let effect_value = signal(0);
    let _effect = Effect::new(move || {
        println!("Effect triggered! Current value: {}", effect_value.get());
    });

    effect_value.set(1);
    effect_value.set(2);

    // 4. Watch
    println!("\n4. Watch:");
    let watch_value = signal("hello");
    let _watch = Watch::new(watch_value.clone(), |value| {
        println!("Watch triggered! Value changed to: {}", value);
    });

    watch_value.set("world");

    // 5. Batch Updates
    println!("\n5. Batch Updates:");
    let batch = Batch::new();
    let a = signal(1);
    let b = signal(2);

    println!("Before batch: a={}, b={}", a.get(), b.get());

    batch.batch(|| {
        a.set(10);
        b.set(20);
        println!("Inside batch: a={}, b={}", a.get(), b.get());
    });

    println!("After batch: a={}, b={}", a.get(), b.get());

    println!("\n✅ Demo completed successfully!");
}