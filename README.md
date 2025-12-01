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

## � Project Structure

This is a monorepo containing all Vela tooling:

```
vela/
├── compiler/           # Vela compiler (lexer, parser, semantic analyzer, codegen)
├── vm/                 # VelaVM (bytecode interpreter)
├── stdlib/             # Standard library
├── cli/                # Vela CLI tool
├── lsp/                # Language Server Protocol implementation
├── devtools/           # DevTools (UI Inspector, Signal Graph, Profiler)
├── docs/               # Documentation
│   ├── architecture/   # ADRs (Architecture Decision Records)
│   ├── specifications/ # Formal specifications
│   ├── tooling/        # Tooling architecture docs
│   └── features/       # Feature documentation
├── tests/              # Test suites
│   ├── unit/           # Unit tests
│   ├── integration/    # Integration tests
│   └── e2e/            # End-to-end tests
├── .github/            # GitHub Actions workflows
├── Cargo.toml          # Rust workspace configuration
├── LICENSE-APACHE      # Apache 2.0 license
├── LICENSE-MIT         # MIT license (dual license)
└── README.md           # This file
```

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ (stable)
- **LLVM** 17+ (for native compilation)
- **Node.js** 18+ (for DevTools UI)

### Installation

```bash
# Clone repository
git clone https://github.com/velalang/vela.git
cd vela

# Build all components
cargo build --release

# Install CLI globally
cargo install --path cli

# Verify installation
vela --version
```

### Hello World

Create `hello.vela`:

```vela
fn main() {
    println("Hello, Vela!");
}
```

Run:

```bash
vela run hello.vela
```

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
# All tests
vela test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
```

### Development Workflow

1. **Create branch:** `git checkout -b feature/VELA-XXX-descripcion`
2. **Make changes**
3. **Run tests:** `cargo test`
4. **Format:** `cargo fmt`
5. **Lint:** `cargo clippy`
6. **Commit:** `git commit -m "feat(VELA-XXX): add feature"`
7. **Push:** `git push origin feature/VELA-XXX-descripcion`
8. **Create PR**

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

**Current Phase:** Phase 0 (Foundation)

| Component | Status | Progress |
|-----------|--------|----------|
| **Sprint 0: Critical Decisions** | ✅ Complete | 100% |
| **Sprint 1: Formal Specifications** | ✅ Complete | 100% |
| **Sprint 2: Tooling Architecture** | ✅ Complete | 100% |
| **Sprint 3: Infrastructure Setup** | 🚧 In Progress | 75% |
| **Compiler (Lexer)** | ⏳ Planned | 0% |
| **Compiler (Parser)** | ⏳ Planned | 0% |
| **Type System** | ⏳ Planned | 0% |
| **Reactive Engine** | ⏳ Planned | 0% |
| **VM** | ⏳ Planned | 0% |
| **Standard Library** | ⏳ Planned | 0% |
| **CLI** | ⏳ Planned | 0% |
| **LSP** | ⏳ Planned | 0% |
| **DevTools** | ⏳ Planned | 0% |

---

## 🗓️ Roadmap

### Phase 0: Foundation (Current)
- ✅ Architectural decisions
- ✅ Formal specifications
- ✅ Tooling architecture
- 🚧 Infrastructure setup

### Phase 1: Core Language (Q2 2026)
- Lexer and parser
- Type system with inference
- Semantic analyzer
- Basic code generation

### Phase 2: Reactive System (Q3 2026)
- Signals, Computed, Effects
- Reactive scheduler
- Dependency tracking

### Phase 3: Multi-Target Compilation (Q4 2026)
- LLVM backend (native)
- JavaScript/WASM backend
- Mobile targets (iOS, Android)

### Vela 1.0 (Q1 2027)
- Stable language spec
- Complete standard library
- Production-ready tooling
- Full documentation

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

*Last updated: 2025-11-30*
