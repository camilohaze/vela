# ADR-003: Estrategia de Testing para Backend Nativo LLVM

## Estado
✅ Aceptado

## Fecha
2025-12-14

## Contexto
Para asegurar la calidad y confiabilidad del backend nativo LLVM de Vela, necesitamos una estrategia completa de testing que valide:

1. **Correctness**: Que el código generado produzca resultados correctos
2. **Performance**: Que las optimizaciones mejoren el rendimiento según lo esperado
3. **Reliability**: Que el backend maneje casos edge y errores gracefully
4. **Integration**: Que todo el pipeline funcione end-to-end
5. **Cross-platform**: Que funcione consistentemente en diferentes plataformas

El testing debe ser automatizado, rápido de ejecutar, y proporcionar feedback temprano en el desarrollo.

## Decisión
Implementar una suite completa de tests en `tests/native_backend/` que incluya:

- **Tests de correctness** para validar funcionalidad básica
- **Tests de performance** con benchmarks comparativos
- **Tests de integración** end-to-end
- **Tests de edge cases** para casos límite
- **Cobertura de código** >= 90%
- **CI/CD integration** con ejecución automática

## Consecuencias

### Positivas
- **Confianza en el código**: Tests exhaustivos aseguran calidad del backend
- **Detección temprana de bugs**: Tests automatizados capturan regresiones
- **Performance validation**: Benchmarks verifican efectividad de optimizaciones
- **Documentación viva**: Tests sirven como ejemplos de uso
- **Facilitan refactoring**: Tests permiten cambios seguros al código
- **CI/CD ready**: Tests se ejecutan automáticamente en pipelines

### Negativas
- **Tiempo de desarrollo**: Implementar tests toma tiempo adicional
- **Complejidad de testing nativo**: Tests requieren compilación y ejecución de binarios
- **Dependencias externas**: Tests necesitan LLVM instalado
- **Mantenimiento**: Tests deben actualizarse cuando cambia la API

## Alternativas Consideradas

### 1. Tests solo en bytecode VM
**Descripción**: Testear solo el generador LLVM sin ejecutar código nativo
**Ventajas**: Más rápido, no requiere LLVM runtime
**Desventajas**: No valida el código nativo final, misses linking/runtime issues
**Rechazada porque**: Necesitamos validar el ejecutable final completo

### 2. Tests manuales únicamente
**Descripción**: Solo testing manual por desarrolladores
**Ventajas**: Simple inicialmente
**Desventajas**: No escalable, propenso a errores humanos, difícil de mantener
**Rechazada porque**: Necesitamos automatización para calidad consistente

### 3. Tests únicamente de integración
**Descripción**: Solo tests end-to-end sin tests unitarios detallados
**Ventajas**: Cubre el flujo completo
**Desventajas**: Difícil debuggear fallos, lento feedback
**Rechazada porque**: Necesitamos tests granulares para debugging rápido

## Implementación

### Arquitectura de Tests

#### Test Framework Structure
```
tests/native_backend/
├── mod.rs              # Common utilities and setup
├── correctness.rs      # Functional correctness tests
├── performance.rs      # Benchmark tests
├── edge_cases.rs       # Edge case and error handling
├── integration.rs      # End-to-end pipeline tests
├── utils.rs           # Testing utilities and helpers
└── fixtures/          # Test data and Vela code samples
    ├── arithmetic.vela
    ├── control_flow.vela
    ├── objects.vela
    └── runtime.vela
```

#### Core Testing Components

**NativeBackendTester** - Main testing harness:
```rust
pub struct NativeBackendTester {
    temp_dir: TempDir,
    llvm_context: Option<Context>,
    runtime_built: bool,
}

impl NativeBackendTester {
    pub fn compile_and_run(&self, vela_code: &str, opt_level: OptimizationLevel) -> Result<TestResult, TestError>
    pub fn benchmark(&self, vela_code: &str, iterations: usize) -> BenchmarkResult
    pub fn validate_correctness(&self, vela_code: &str, expected_output: &str) -> bool
}
```

**TestResult** - Execution result validation:
```rust
pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub peak_memory: usize,
    pub compilation_time: Duration,
}
```

### Tipos de Tests

#### 1. Correctness Tests
**Propósito**: Validar que el código generado produce resultados correctos
**Ejemplos**:
- Operaciones aritméticas básicas
- Control flow (if/else, loops)
- Funciones y recursión
- Arrays y objetos
- Runtime operations (GC, signals, actors)

```rust
#[test]
fn test_arithmetic_expressions() {
    let vela_code = r#"
        fn main() -> void {
            let result = (10 + 5) * 3 - 7;
            print(result);  // Should print 38
        }
    "#;

    let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)?;
    assert_eq!(result.stdout.trim(), "38");
    assert_eq!(result.exit_code, 0);
}
```

#### 2. Performance Tests
**Propósito**: Validar que las optimizaciones mejoran el rendimiento
**Métricas**:
- Compilation time por nivel de optimización
- Execution time para diferentes inputs
- Memory usage con GC
- Binary size

```rust
#[test]
fn test_optimization_speedup() {
    let compute_intensive_code = r#"
        fn fibonacci(n: int) -> int {
            if n <= 1 { return n; }
            return fibonacci(n-1) + fibonacci(n-2);
        }

        fn main() -> void {
            let result = fibonacci(35);
            print(result);
        }
    "#;

    let benchmark = tester.benchmark(compute_intensive_code, 5)?;

    // O3 should be at least 2x faster than O0
    assert!(benchmark.speedup_o3 > 2.0);
    // Each optimization level should improve performance
    assert!(benchmark.o3_time < benchmark.o2_time);
    assert!(benchmark.o2_time < benchmark.o1_time);
}
```

#### 3. Edge Case Tests
**Propósito**: Validar manejo robusto de casos límite
**Casos**:
- Stack overflow con recursión profunda
- Memory exhaustion
- Type limits (integer overflow, float precision)
- Error conditions (division by zero, null pointer access)

```rust
#[test]
fn test_stack_overflow_handling() {
    let deep_recursion_code = r#"
        fn recursive_func(n: int) -> int {
            if n <= 0 { return 0; }
            return recursive_func(n - 1) + 1;
        }

        fn main() -> void {
            let result = recursive_func(10000);
            print(result);
        }
    "#;

    // Should either complete successfully or fail gracefully
    let result = tester.compile_and_run_with_timeout(
        deep_recursion_code,
        OptimizationLevel::Default,
        Duration::from_secs(30)
    );

    match result {
        Ok(test_result) => {
            // If it completes, result should be correct
            assert_eq!(test_result.stdout.trim(), "10000");
        }
        Err(TestError::Timeout) => {
            // Timeout is acceptable for deep recursion
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
```

#### 4. Integration Tests
**Propósito**: Validar el pipeline completo end-to-end
**Cobertura**:
- Parser → IR generation
- IR → LLVM translation
- LLVM optimization
- Object file generation
- Linking with runtime
- Executable validation

```rust
#[test]
fn test_full_pipeline_integration() {
    let complex_vela_program = r#"
        // Complex program using multiple features
        struct Person {
            name: string,
            age: int,
        }

        fn create_person(name: string, age: int) -> Person {
            return Person { name: name, age: age };
        }

        fn main() -> void {
            let people = [];
            for i in 0..5 {
                let person = create_person("Person_${i}", 20 + i);
                people.push(person);
            }

            for person in people {
                print("${person.name} is ${person.age} years old");
            }
        }
    "#;

    // Test compilation
    let compile_result = tester.compile_to_executable(
        complex_vela_program,
        OptimizationLevel::Default
    )?;
    assert!(compile_result.executable_path.exists());

    // Test execution
    let run_result = tester.run_executable(&compile_result.executable_path)?;
    assert_eq!(run_result.exit_code, 0);
    assert!(run_result.stdout.contains("Person_0 is 20 years old"));
    assert!(run_result.stdout.contains("Person_4 is 24 years old"));
}
```

### Testing Infrastructure

#### Build and Execution Environment
```rust
pub struct TestEnvironment {
    pub llvm_available: bool,
    pub cmake_available: bool,
    pub compiler_available: bool,
    pub temp_directory: TempDir,
}

impl TestEnvironment {
    pub fn setup() -> Result<Self, TestError> {
        // Check for required tools
        let llvm_available = check_llvm_installation()?;
        let cmake_available = check_cmake_installation()?;
        let compiler_available = check_compiler_installation()?;

        Ok(TestEnvironment {
            llvm_available,
            cmake_available,
            compiler_available,
            temp_directory: TempDir::new()?,
        })
    }
}
```

#### Test Configuration
```toml
[package.metadata.native-backend-tests]
# Test timeouts
correctness_timeout = "30s"
performance_timeout = "300s"
integration_timeout = "60s"

# Memory limits
max_memory_per_test = "100MB"

# Benchmark settings
benchmark_iterations = 10
warmup_iterations = 3
```

### CI/CD Integration

#### GitHub Actions Configuration
```yaml
name: Native Backend Tests

on:
  push:
    paths:
      - 'compiler/src/codegen/**'
      - 'runtime/**'
      - 'tests/native_backend/**'

jobs:
  test-native-backend:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - name: Setup LLVM
        uses: KyleMayes/install-llvm-action@v1
        with:
          version: "17.0"

      - name: Run native backend tests
        run: cargo test --features llvm_backend --test native_backend

      - name: Run benchmarks
        run: cargo bench --features llvm_backend
```

### Métricas de Calidad

#### Coverage Requirements
- **Line coverage**: >= 90% para código del backend nativo
- **Branch coverage**: >= 85% para lógica condicional
- **Function coverage**: 100% para APIs públicas

#### Performance Baselines
- **Test execution time**: < 5 minutos para suite completa
- **Compilation time**: < 30 segundos por test típico
- **Memory usage**: < 100MB por test
- **Binary size**: < 5MB para ejecutables de test

## Referencias
- **Jira:** [VELA-1123](https://velalang.atlassian.net/browse/VELA-1123)
- **TASK:** [TASK-126](https://velalang.atlassian.net/browse/TASK-126)
- **Dependencias:** TASK-121, TASK-122, TASK-123, TASK-124, TASK-125 completadas
- **Documentación:** Ver `docs/features/VELA-1123/TASK-126.md`

## Implementación
Ver código en: `tests/native_backend/`