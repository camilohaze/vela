# Troubleshooting Guide

## 🐛 Common Compilation Errors

### Syntax Errors

#### "Expected expression, found '}'"

**Error:**
```
error: expected expression, found '}'
  --> source.vela:5:5
   |
 5 |     }
   |     ^ expected expression
```

**Cause:** Missing return statement or expression in function.

**Fix:**
```vela
// ❌ Wrong
fn add(a: Number, b: Number) -> Number {
    // Missing return
}

// ✅ Correct
fn add(a: Number, b: Number) -> Number {
    return a + b
}

// Or use expression body
fn add(a: Number, b: Number) -> Number {
    a + b  // Implicit return
}
```

#### "Undefined variable"

**Error:**
```
error: undefined variable 'undefined_var'
  --> source.vela:10:5
   |
10 | print(undefined_var)
   |      ^^^^^^^^^^^^^^ variable not defined in this scope
```

**Cause:** Variable used before declaration or out of scope.

**Fix:**
```vela
// ✅ Declare before use
let message = "Hello"
print(message)

// ✅ Check scope
fn test() {
    let local_var = "local"
    print(local_var)  // OK
}
// print(local_var)  // Error: out of scope
```

### Type Errors

#### "Type mismatch"

**Error:**
```
error: type mismatch
  --> source.vela:8:9
   |
 8 | let x: Number = "hello"
   |         ^^^^^^ expected Number, found String
```

**Cause:** Assigning incompatible type.

**Fix:**
```vela
// ✅ Correct types
let x: Number = 42
let y: String = "hello"

// ✅ Type conversion
let x: Number = "42".to_number().unwrap_or(0)
```

#### "Property does not exist"

**Error:**
```
error: property 'length' does not exist on type 'Number'
  --> source.vela:3:12
   |
 3 | let len = 42.length()
   |            ^^^^^^^^^^^
```

**Cause:** Calling method that doesn't exist on type.

**Fix:**
```vela
// ✅ Check available methods
let num: Number = 42
let str: String = num.to_string()
let len = str.length()  // OK
```

### Semantic Errors

#### "Duplicate definition"

**Error:**
```
error: duplicate definition of 'test'
  --> source.vela:3:4
   |
 3 | fn test() {}
   |    ^^^^ already defined at source.vela:1:4
```

**Cause:** Same name defined multiple times in same scope.

**Fix:**
```vela
// ✅ Use unique names
fn test_v1() {}
fn test_v2() {}

// ✅ Use modules
module Math {
    fn add() {}
}

module StringUtils {
    fn add() {}  // OK, different module
}
```

#### "Unreachable code"

**Warning:**
```
warning: unreachable code
  --> source.vela:6:5
   |
 6 |     print("This never executes")
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ code after return is unreachable
```

**Cause:** Code after return, break, or throw.

**Fix:**
```vela
// ✅ Remove unreachable code
fn test() -> Number {
    return 42
    // print("unreachable")  // Remove this
}

// ✅ Use early return pattern
fn process(data: String) -> Result<String, Error> {
    if data.is_empty() {
        return Err(Error("Empty data"))
    }

    // Continue processing...
    return Ok(data.to_uppercase())
}
```

## 🔧 Runtime Issues

### Stack Overflow

**Error:** Program crashes with stack overflow.

**Cause:** Infinite recursion or very deep recursion.

**Fix:**
```vela
// ❌ Infinite recursion
fn loop_forever() {
    loop_forever()  // Stack overflow!
}

// ✅ Use iteration or tail recursion
fn sum_list(list: [Number]) -> Number {
    match list {
        [] => 0
        [head, ...tail] => head + sum_list(tail)  // Tail recursive
    }
}

// ✅ Use loops
fn sum_list_iterative(list: [Number]) -> Number {
    let sum = 0
    list.forEach(x => sum = sum + x)  // Not tail recursive but iterative
    return sum
}
```

### Out of Memory

**Error:** Program runs out of memory.

**Cause:** Creating too many objects or large data structures.

**Fix:**
```vela
// ✅ Use streaming for large data
fn process_large_file(path: String) -> Result<void, Error> {
    return File.open(path)
        .and_then(file => file.lines()
            .forEach(line => process_line(line)))
}

// ✅ Limit collection sizes
fn collect_valid_items(items: [String]) -> [String] {
    return items
        .filter(item => is_valid(item))
        .take(1000)  // Limit to prevent OOM
}
```

## 🛠️ Tool Issues

### Compiler Won't Start

**Problem:** `vela-compiler` command not found.

**Solutions:**

1. **Check PATH:**
```bash
# Add to PATH
export PATH=$PATH:/path/to/vela-compiler

# Or create symlink
sudo ln -s /path/to/vela-compiler /usr/local/bin/vela-compiler
```

2. **Check permissions:**
```bash
chmod +x /path/to/vela-compiler
```

3. **Reinstall:**
```bash
cargo install --path crates/vela-compiler --force
```

### Compilation is Slow

**Problem:** Compilation takes too long.

**Solutions:**

1. **Use optimizations:**
```bash
vela-compiler compile source.vela -O2  # Faster than -O0
```

2. **Parallel compilation:**
```bash
vela-compiler compile src/ -j 8  # Use 8 threads
```

3. **Incremental builds:**
```bash
vela-compiler compile --incremental source.vela
```

4. **Profile compilation:**
```bash
vela-compiler compile --profile source.vela
# Check vela-compiler-profile.json for bottlenecks
```

### Large Binary Size

**Problem:** Compiled bytecode is too large.

**Solutions:**

1. **Strip debug info:**
```bash
vela-compiler compile source.vela --release  # No debug info
```

2. **Use compression:**
```bash
vela-compiler compile source.vela --compress
```

3. **Dead code elimination:**
```bash
vela-compiler compile source.vela --optimize dead-code
```

## 🔍 Debugging Techniques

### Using the REPL

```bash
# Start REPL
vela-compiler repl

# Test expressions
> 1 + 2
3

> let x = 42
> x * 2
84

# Test functions
> fn test(a, b) { a + b }
> test(5, 3)
8
```

### Debug Logging

```bash
# Enable debug logging
vela-compiler compile --debug source.vela

# View detailed compilation steps
vela-compiler compile --verbose source.vela
```

### Source Maps

```bash
# Generate source map
vela-compiler compile --source-map source.vela

# Use in debugger
vela-vm debug program.bytecode --source-map program.map
```

### Profiling

```bash
# Profile execution
vela-vm run --profile program.bytecode > profile.json

# Analyze profile
vela-profiler analyze profile.json
```

## 🌐 Platform-Specific Issues

### Windows

**Path separator issues:**
```bash
# Use forward slashes or escaped backslashes
import "src/utils"        # ✅
import "src\\utils"       # ✅
# import "src\utils"      # ❌ (escape sequence)
```

**File permissions:**
```bash
# Run as administrator or check permissions
icacls "program.bytecode" /grant Users:F
```

### macOS/Linux

**Library path issues:**
```bash
# Set library path
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib
```

**Case sensitivity:**
```vela
// ✅ Case sensitive
import "MyModule"

// ❌ Wrong case
// import "mymodule"  // Different from MyModule
```

### WebAssembly

**Unsupported features:**
- File system access
- Network operations
- Some system calls

**Fix:**
```vela
// Check platform at compile time
if Platform.is_wasm() {
    // Use web-compatible APIs
} else {
    // Use native APIs
}
```

## 📊 Performance Issues

### Slow Execution

**Common causes:**

1. **Excessive allocations:**
```vela
// ❌ Creates many temporary strings
let result = ""
list.forEach(item => result = result + item)

// ✅ Use efficient concatenation
let result = list.join("")
```

2. **Inefficient algorithms:**
```vela
// ❌ O(n²)
let duplicates = []
list.forEach(a =>
    list.forEach(b =>
        if a == b { duplicates.push(a) }))

// ✅ O(n) with Set
let seen = Set()
let duplicates = list.filter(item => {
    if seen.has(item) {
        return true
    }
    seen.add(item)
    return false
})
```

3. **Unnecessary computations:**
```vela
// ❌ Recomputes every time
let expensive = compute_expensive()
if condition {
    print(expensive)
}

// ✅ Compute only when needed
if condition {
    let expensive = compute_expensive()
    print(expensive)
}
```

### Memory Leaks

**In Vela, memory is managed automatically, but:**

1. **Circular references:**
```vela
// ❌ Circular reference prevents GC
class A { b: B }
class B { a: A }
let a = A()
let b = B()
a.b = b
b.a = a  // Circular reference
```

2. **Large collections:**
```vela
// ✅ Clear large collections when done
let large_list = load_large_data()
process_data(large_list)
large_list.clear()  // Free memory
```

## 🔗 Integration Issues

### With Build Systems

#### Cargo (Rust projects)

**Mixed Rust/Vela project:**
```toml
[dependencies]
vela-compiler = "1.0"

[build-dependencies]
vela-compiler = "1.0"
```

**Build script issues:**
```rust
// build.rs
use std::process::Command;

fn main() {
    // Ensure vela-compiler is available
    match Command::new("vela-compiler").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("Vela compiler found");
        }
        _ => {
            panic!("Vela compiler not found. Install with: cargo install vela-compiler");
        }
    }
}
```

#### Make

**Makefile issues:**
```makefile
VELA_COMPILER := vela-compiler

$(BUILD_DIR)/%.bytecode: $(SRC_DIR)/%.vela
    @$(VELA_COMPILER) --version > /dev/null || (echo "Install vela-compiler: cargo install vela-compiler" && exit 1)
    $(VELA_COMPILER) compile $< -o $@
```

### IDE Integration

#### VS Code

**Extension not working:**
1. Check extension is installed: `Vela Language Support`
2. Reload window: `Ctrl+Shift+P` → `Developer: Reload Window`
3. Check settings:
```json
{
    "vela.compiler.path": "vela-compiler",
    "vela.compiler.enableDiagnostics": true
}
```

#### Vim/Neovim

**Syntax highlighting not working:**
```vim
" Check plugin is installed
:scriptnames | grep vela

" Reinstall plugin
:PlugInstall
```

## 📞 Getting Help

### Before Asking

1. **Check version:**
```bash
vela-compiler --version
vela-vm --version
```

2. **Update tools:**
```bash
cargo install --force vela-compiler
cargo install --force vela-vm
```

3. **Minimal reproduction:**
Create the smallest possible code that reproduces the issue.

### Where to Ask

1. **GitHub Issues:** Bug reports and feature requests
2. **GitHub Discussions:** Questions and help
3. **Stack Overflow:** Tag with `vela-lang`
4. **Discord/Slack:** Community chat

### What to Include

When reporting issues, include:

- **Version information:**
```bash
vela-compiler --version
rustc --version
cargo --version
```

- **Platform:** Windows/macOS/Linux, version, architecture

- **Minimal code sample** that reproduces the issue

- **Expected vs actual behavior**

- **Error messages** (full output)

- **Steps to reproduce**

---

*Documentación generada automáticamente. Última actualización: 2025-12-03*