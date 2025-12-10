# Vela Programming Language

**Version:** 0.1.0 (Phase 0 - Pre-Alpha)  
**Status:** 🚧 Under Active Development  
**License:** Apache 2.0 / MIT (dual license)

---

## 🎯 What is Vela?

**Vela** is a modern, reactive programming language designed for building cross-platform applications with a focus on:

- ✨ **Reactive-first**: Built-in signals and computed values for automatic UI updates
- 🚀 **Multi-target**: Compile to VM bytecode, native binaries, JavaScript/WASM, mobile, and desktop
- 🔒 **Memory-safe**: Automatic Reference Counting (ARC) with cycle detection
- 🎨 **Declarative UI**: Widget-based UI framework inspired by Flutter and SwiftUI
- ⚡ **High-performance**: LLVM-based native compilation with zero-cost abstractions
- 🧩 **Dependency Injection**: Built-in DI system for clean architecture

---

## 📁 Project Structure

This is a monorepo containing all Vela tooling implemented in Rust:

```
vela/
├── core/                         # Core types, AST, IR (Rust)
├── compiler/                     # Compiler: lexer, parser, semantic analyzer, codegen (Rust)
├── vm/                           # Virtual Machine: bytecode interpreter (Rust + Vela)
├── runtime/                      # Runtime system: reactive, concurrency, GC (Rust)
├── stdlib/                       # Standard library (Rust + Vela bindings)
├── tooling/                      # Development tools: CLI, LSP, debugger, devtools (Rust)
├── packages/                     # Additional packages and systems (Rust)
│   ├── concurrency/              # Advanced concurrency system (actors, channels)
│   ├── devtools/                 # DevTools (UI Inspector, Signal Graph, Profiler)
│   ├── di/                       # Dependency Injection system
│   ├── docs/                     # Documentation generation tools
│   ├── events/                   # Event system and pub/sub
│   ├── http/                     # HTTP client/server framework
│   ├── i18n/                     # Internationalization system
│   ├── logging/                  # Async logging with transports and filtering
│   ├── lsp/                      # Language Server Protocol implementation
│   ├── package/                  # Package manager and resolver
│   ├── reactive/                 # Reactive programming primitives (signals, computed)
│   ├── state-management/         # State management (Redux-like with middleware)
│   ├── ui/                       # UI framework (widgets, rendering, styling)
│   └── validation/               # Validation system and decorators
├── bin/                          # Executable binaries
├── benches/                      # Performance benchmarks
├── examples/                     # Example applications and code samples
│   ├── ui/                       # UI framework examples
│   ├── hello-world/              # Basic "Hello World" example
│   └── ...                       # Additional examples
├── docs/                         # Documentation
│   ├── architecture/             # Architecture Decision Records (ADRs)
│   ├── features/                 # Feature documentation by user story
│   ├── api/                      # API specifications
│   └── design/                   # Design documents and diagrams
├── tests/                        # Test suites
│   ├── unit/                     # Unit tests (Rust + Vela source files)
│   ├── integration/              # Integration tests
│   └── benchmarks/               # Benchmark tests
├── jira-import/                  # Jira import and management scripts
├── .github/                      # GitHub Actions workflows and templates
├── Cargo.toml                    # Rust workspace configuration
├── Cargo.lock                    # Dependency lock file
├── LICENSE-APACHE                # Apache 2.0 license
├── LICENSE-MIT                   # MIT license (dual license)
└── README.md                     # This file
```

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ (stable toolchain)
- **Cargo** (included with Rust)
- **Git** for version control

### Installation

```bash
# Clone repository
git clone https://github.com/camilohaze/vela.git
cd vela

# Build all components
cargo build --release

# Run tests to verify installation
cargo test

# (Optional) Install development tools
cargo install cargo-watch  # For auto-rebuilding
cargo install cargo-tarpaulin  # For test coverage
```

### Hello World

Create `examples/hello.vela`:

```vela
fn main() {
    println("Hello, Vela! 🦀");
}
```

Currently, Vela is in early development. The compiler and runtime are being actively developed. Check the [examples/](examples/) directory for sample code.

---

## 📚 Documentation

- **Language Reference:** [docs/specifications/vela-language-specification.md](docs/specifications/vela-language-specification.md)
- **Getting Started:** [docs.velalang.org/getting-started](https://docs.velalang.org/getting-started) (coming soon)
- **API Reference:** [docs.velalang.org/api](https://docs.velalang.org/api) (coming soon)
- **Architecture Decisions:** [docs/architecture/](docs/architecture/)

---

## 🛠️ Development

### Building from Source

```bash
# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Running Tests

```bash
# All tests across the workspace
cargo test --workspace

# Run specific package tests
cargo test -p vela_compiler
cargo test -p vela_vm
cargo test -p vela_lsp

# Run with verbose output
cargo test --workspace --verbose

# Run benchmarks
cargo bench

# Generate test coverage (requires tarpaulin)
cargo tarpaulin --workspace --out Html
```

### Development Workflow

1. **Create feature branch:** `git checkout -b feature/VELA-XXX-description`
2. **Make changes** following the established patterns
3. **Run tests:** `cargo test --workspace`
4. **Format code:** `cargo fmt`
5. **Lint:** `cargo clippy`
6. **Commit:** `git commit -m "feat(VELA-XXX): description"`
7. **Push:** `git push origin feature/VELA-XXX-description`
8. **Create PR** with proper description and Jira links

---

## 🤝 Contributing

We welcome contributions! Please read our [CONTRIBUTING.md](.github/CONTRIBUTING.md) for:

- Code of Conduct
- Development setup
- Coding standards
- PR process
- Testing guidelines

---

## 📊 Project Status

**Current Phase:** Phase 0 (Foundation) - Sprint 16+  
**Version:** 0.1.0 (Pre-Alpha)  
**Implementation Language:** Rust 🦀

| Component | Status | Progress | Tests | Sprint |
|-----------|--------|----------|-------|--------|
| **Critical Decisions** | ✅ Complete | 100% | - | Sprint 0 |
| **Formal Specifications** | ✅ Complete | 100% | - | Sprint 1 |
| **Tooling Architecture** | ✅ Complete | 100% | - | Sprint 2 |
| **Infrastructure Setup** | ✅ Complete | 100% | - | Sprint 3 |
| **Language Grammar (EBNF)** | ✅ Complete | 100% | - | Sprint 4 |
| **Lexer Implementation** | ✅ Complete | 100% | 50+ | Sprint 5 |
| **Parser Implementation** | ✅ Complete | 100% | 80+ | Sprint 6-7 |
| **Type System Design** | ✅ Complete | 100% | - | Sprint 8 |
| **Keyword-Specific Validation** | ✅ Complete | 100% | - | Sprint 9 |
| **Reactive System (Signals)** | ✅ Complete | 100% | 245+ | Sprint 11-12 |
| **Dependency Injection** | ✅ Complete | 100% | 327+ | Sprint 13 |
| **Event System** | ✅ Complete | 100% | 231+ | Sprint 14 |
| **LSP Implementation** | 🚧 In Progress | 60% | 45+ | Sprint 16 |
| **State Management** | ✅ Complete | 100% | 180+ | Sprint 15 |
| **HTTP Framework** | ✅ Complete | 100% | 95+ | Sprint 10 |
| **UI Framework** | 🚧 In Progress | 40% | 120+ | Sprint 17 |
| **Concurrency (Actors)** | ✅ Complete | 100% | 160+ | Sprint 18 |
| **Validation System** | ✅ Complete | 100% | 85+ | Sprint 19 |
| **Package Manager** | 🚧 In Progress | 30% | 60+ | Sprint 20 |
| **Standard Library** | 🚧 In Progress | 25% | 90+ | Sprint 21 |
| **VM Implementation** | ⏳ Planned | 0% | - | Sprint 22+ |
| **Code Generation** | ⏳ Planned | 0% | - | Sprint 25+ |

**Total Tests Passing:** 1,200+ tests across all systems  
**Total LOC (Production + Tests):** ~45,000 LOC  
**Architecture:** Monorepo with 15+ Rust crates

---

## 🗓️ Roadmap

### Phase 0: Foundation (Current - Sprint 16+)
- ✅ **Completed Systems:**
  - Critical architectural decisions (Sprint 0)
  - Formal language specifications (Sprint 1)
  - Tooling architecture design (Sprint 2)
  - Infrastructure setup (Sprint 3)
  - Language grammar & EBNF (Sprint 4)
  - Complete lexer implementation (Sprint 5)
  - Parser with AST generation (Sprint 6-7)
  - Type system design (Sprint 8)
  - Keyword-specific validation (Sprint 9)
  - HTTP framework (Sprint 10)
  - Reactive signals system (Sprint 11-12)
  - Dependency injection (Sprint 13)
  - Event system (Sprint 14)
  - State management (Sprint 15)
  - LSP implementation (Sprint 16 - 60% complete)
  - UI framework foundation (Sprint 17 - 40% complete)
  - Actor-based concurrency (Sprint 18)
  - Validation system (Sprint 19)
  - Package manager (Sprint 20 - 30% complete)
  - Standard library (Sprint 21 - 25% complete)

### Phase 1: Core Language (Q1 2026)
- VM bytecode interpreter implementation
- Code generation backends (LLVM, WASM, JS)
- Complete semantic analyzer
- Advanced type system features
- Memory management (ARC + GC)

### Phase 2: Multi-Target Compilation (Q2 2026)
- Native binary compilation (x86_64, ARM64)
- WebAssembly backend for browsers
- JavaScript transpilation
- Mobile targets (iOS/Android via bindings)

### Phase 3: Ecosystem & Tooling (Q3 2026)
- Complete DevTools suite
- Package registry infrastructure
- IDE integrations (VS Code, IntelliJ)
- Performance profiling tools
- Documentation generation

### Vela 1.0 (Q4 2026)
- Stable language specification
- Production-ready compiler and tooling
- Complete standard library
- Comprehensive documentation
- Community ecosystem established

---

## � License

Vela is dual-licensed under:

- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

You may choose either license for your use.

---

## 🔗 Links

- **Website:** [velalang.org](https://velalang.org) (coming soon)
- **Documentation:** [docs.velalang.org](https://docs.velalang.org) (coming soon)
- **GitHub:** [github.com/velalang/vela](https://github.com/velalang/vela)
- **Jira:** [velalang.atlassian.net](https://velalang.atlassian.net)
- **Discord:** [discord.gg/vela](https://discord.gg/vela) (coming soon)
- **Twitter:** [@velalang](https://twitter.com/velalang) (coming soon)

---

## � Philosophy

Vela is built on these core principles:

1. **Simplicity over complexity:** Easy things should be easy, hard things should be possible
2. **Safety by default:** Memory-safe, null-safe, thread-safe
3. **Performance without compromise:** Zero-cost abstractions, LLVM optimization
4. **Developer experience:** Great error messages, powerful tooling, fast compile times
5. **Cross-platform from day one:** Write once, run anywhere (native, web, mobile)

---

## � Acknowledgments

Vela is inspired by and builds upon ideas from:

- **Rust** - Memory safety, ownership, zero-cost abstractions
- **Swift** - Clean syntax, ARC, protocol-oriented programming
- **Kotlin** - Null-safety, extension functions, coroutines
- **Flutter** - Declarative UI, hot reload
- **SolidJS** - Fine-grained reactivity with signals
- **TypeScript** - Structural type system, gradual typing

---

## 📧 Contact

For questions, suggestions, or feedback:

- **Email:** hello@velalang.org
- **GitHub Issues:** [github.com/velalang/vela/issues](https://github.com/velalang/vela/issues)

---

**Made with ❤️ by the Vela Core Team**

---

*Last updated: 2025-12-02*
