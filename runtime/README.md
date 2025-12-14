# Vela Runtime Library

The Vela Runtime Library provides the runtime support needed for native execution of Vela programs compiled to LLVM IR. It implements garbage collection, reactive signals, and actor-based concurrency.

## Overview

This runtime library is part of TASK-123 in VELA-620 (Backend Native - LLVM). It provides:

- **Garbage Collector**: Mark-and-sweep GC for automatic memory management
- **Reactive Signals**: Automatic dependency tracking and change propagation
- **Actor System**: Message-passing concurrency with isolated processes

## Architecture

```
runtime/
├── include/           # Public headers
│   ├── vela_runtime.h # Main API
│   ├── gc.h          # GC internals
│   ├── signals.h     # Signals internals
│   └── actors.h      # Actors internals
├── src/              # Implementation
│   ├── gc.c          # GC implementation
│   ├── signals.c     # Signals implementation
│   ├── actors.c      # Actors implementation
│   └── runtime.c     # Main runtime + Vela objects
├── CMakeLists.txt    # Build system
└── README.md         # This file
```

## Building

### Prerequisites

- CMake 3.10 or later
- C compiler (GCC, Clang, or MSVC)
- POSIX threads (pthreads) - usually included with the compiler

### Build Steps

```bash
# Create build directory
mkdir build
cd build

# Configure with CMake
cmake ..

# Build
make

# Install (optional)
make install
```

### Build Options

- `BUILD_EXAMPLES=ON`: Build example programs
- `BUILD_TESTS=ON`: Build and run tests

## Usage

### Linking with LLVM Backend

The runtime library is designed to be linked with LLVM-generated native code. Include the headers and link against the library:

```c
#include <vela_runtime.h>

// In your LLVM-generated code
int main() {
    // Initialize runtime
    vela_runtime_init();

    // Your Vela program code here
    // ...

    // Cleanup
    vela_runtime_shutdown();
    return 0;
}
```

### Compilation

```bash
# Static linking
gcc -o program program.o -L/path/to/runtime/lib -lvela_runtime -lpthread

# Dynamic linking
gcc -o program program.o -L/path/to/runtime/lib -lvela_runtime -lpthread -Wl,-rpath,/path/to/runtime/lib
```

## API Reference

### Initialization

```c
void vela_runtime_init(void);      // Initialize all subsystems
void vela_runtime_shutdown(void);  // Shutdown all subsystems
const char* vela_runtime_version(void); // Get version string
```

### Garbage Collection

```c
void* vela_gc_alloc(size_t size);  // Allocate GC-managed memory
void vela_gc_collect(void);        // Force garbage collection
void vela_gc_add_root(void* ptr);  // Add to GC root set
void vela_gc_remove_root(void* ptr); // Remove from GC root set
```

### Reactive Signals

```c
vela_signal_t* vela_signal_create(void* initial_value);
void vela_signal_set(vela_signal_t* signal, void* value);
void* vela_signal_get(vela_signal_t* signal);

vela_computed_t* vela_computed_create(vela_compute_fn compute_fn);
void* vela_computed_get(vela_computed_t* computed);
```

### Actor System

```c
vela_actor_t* vela_actor_create(vela_actor_fn behavior, void* initial_state);
bool vela_actor_send(vela_actor_t* actor, vela_message_t* message);
void* vela_actor_get_state(vela_actor_t* actor);
```

### Vela Objects

```c
// Arrays
vela_object_t* vela_array_create(size_t element_count, size_t element_size);
void* vela_array_get(vela_object_t* array, size_t index);
bool vela_array_set(vela_object_t* array, size_t index, void* value);
size_t vela_array_length(vela_object_t* array);

// Strings
vela_object_t* vela_string_create(const char* c_string);
const char* vela_string_get(vela_object_t* string);
size_t vela_string_length(vela_object_t* string);

// Objects
vela_object_t* vela_object_create(void);
bool vela_object_set(vela_object_t* object, const char* key, void* value);
void* vela_object_get(vela_object_t* object, const char* key);
```

## Integration with LLVM Backend

The LLVM IR generator should be modified to:

1. Include runtime headers
2. Generate calls to runtime functions for complex operations
3. Use GC allocation for Vela objects
4. Link against the runtime library

Example LLVM IR integration:

```llvm
; Include runtime declarations
declare void @vela_init_runtime()
declare void @vela_shutdown_runtime()
declare i8* @vela_gc_allocate(i64)
declare i8* @vela_create_array(i64, i64)

; In main function
call void @vela_init_runtime()

; Allocate array
%array = call i8* @vela_create_array(i64 10, i64 8)

; ... program code ...

call void @vela_shutdown_runtime()
```

## Memory Management

The runtime uses a mark-and-sweep garbage collector:

- **Allocation**: Objects are allocated with `vela_gc_alloc()`
- **Root Set**: Pointers in the root set are always considered reachable
- **Mark Phase**: Starting from roots, marks all reachable objects
- **Sweep Phase**: Frees unmarked objects

## Thread Safety

- **GC**: Not thread-safe - should be called from main thread
- **Signals**: Thread-safe for reads/writes
- **Actors**: Designed for concurrent message passing

## Performance Considerations

- GC pauses may occur during collection
- Signal propagation has O(n) complexity where n is dependency depth
- Actor mailboxes have bounded capacity to prevent unbounded growth

## Testing

Build with tests enabled:

```bash
cmake -DBUILD_TESTS=ON ..
make
make test
```

## Examples

See the `examples/` directory for usage examples.

## License

This runtime library is part of the Vela programming language project.
