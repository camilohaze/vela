# The Vela Programming Language

**Welcome to the official Vela documentation!**

---

## What is Vela?

**Vela** is a modern, reactive programming language designed for building cross-platform applications with a focus on:

- ✨ **Reactive-first**: Built-in signals and computed values for automatic UI updates
- 🚀 **Multi-target**: Compile to VM bytecode, native binaries, JavaScript/WASM, mobile, and desktop
- 🔒 **Memory-safe**: Automatic Reference Counting (ARC) with cycle detection
- 🎨 **Declarative UI**: Widget-based UI framework inspired by Flutter and SwiftUI
- ⚡ **High-performance**: LLVM-based native compilation with zero-cost abstractions
- 🧩 **Dependency Injection**: Built-in DI system for clean architecture

---

## Quick Example

```vela
import ui { App, Window, Button, Text }
import reactive { signal }

fn main() {
    let count = signal(0);
    
    App::new()
        .window(Window {
            title: "Counter Example",
            child: Column {
                children: [
                    Text { text: "Count: {count}" },
                    Button {
                        text: "Increment",
                        on_click: || count.update(|c| c + 1)
                    }
                ]
            }
        })
        .run();
}
```

---

## Documentation Structure

This documentation is organized into the following sections:

### 📚 [Getting Started](./getting-started/installation.md)
Learn how to install Vela, set up your development environment, and write your first program.

### 📖 [Language Guide](./language/variables-and-types.md)
Comprehensive guide to the Vela language, covering syntax, types, functions, and more.

### ⚡ [Reactive Programming](./reactive/introduction.md)
Deep dive into Vela's reactive system: signals, computed values, effects, and stores.

### 🎨 [UI Framework](./ui/introduction.md)
Build beautiful cross-platform UIs with Vela's declarative widget-based framework.

### 🔌 [Dependency Injection](./di/introduction.md)
Learn how to use Vela's built-in dependency injection system for clean architecture.

### 🔄 [Asynchronous Programming](./async/introduction.md)
Master async/await, futures, concurrency, and streams in Vela.

### 📦 [Standard Library](./stdlib/overview.md)
Complete reference for Vela's standard library APIs.

### 🚀 [Advanced Topics](./advanced/memory-management.md)
Explore advanced features like generics, metaprogramming, FFI, and performance optimization.

### 🛠️ [Tooling](./tooling/cli.md)
Learn about Vela's CLI, package manager, language server, and DevTools.

### 📱 [Multi-Platform Development](./multi-platform/overview.md)
Build for native, web, iOS, Android, and desktop from a single codebase.

### 💡 [Best Practices](./best-practices/code-style.md)
Guidelines for writing idiomatic, maintainable, and performant Vela code.

### 📋 [Reference](./reference/language-specification.md)
Formal specifications and technical reference documentation.

### 📎 [Appendix](./appendix/glossary.md)
Glossary, cheat sheets, migration guides, and FAQ.

---

## Getting Help

If you need help with Vela:

- 📖 **Documentation**: Start with the [Getting Started](./getting-started/installation.md) guide
- 💬 **Community**: Join our [Discord server](https://discord.gg/vela) (coming soon)
- 🐛 **Bug Reports**: Submit issues on [GitHub](https://github.com/velalang/vela/issues)
- 💡 **Feature Requests**: Use [GitHub Discussions](https://github.com/velalang/vela/discussions)
- 📧 **Email**: Contact us at hello@velalang.org

---

## Contributing

Vela is an open-source project, and we welcome contributions! See our [Contributing Guide](./appendix/contributing.md) to get started.

---

## License

Vela is dual-licensed under:

- **Apache License 2.0**
- **MIT License**

You may choose either license for your use.

---

## Project Status

**Current Phase:** Phase 0 (Foundation)

Vela is under active development. The current version is **0.1.0 (Pre-Alpha)**.

See the [Roadmap](https://github.com/velalang/vela#roadmap) for upcoming features and milestones.

---

**Let's build something amazing with Vela! 🚀**
