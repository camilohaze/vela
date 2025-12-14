# TASK-126: Tests de backend nativo

## 📋 Información General
- **Historia:** VELA-1123
- **Estado:** En curso 🔄
- **Fecha:** 2025-12-14

## 🎯 Objetivo
Implementar una suite completa de tests para validar el correcto funcionamiento del backend nativo LLVM de Vela, incluyendo tests de correctness, performance y edge cases.

## 🔨 Implementación

### Arquitectura de Tests

El módulo de tests `tests/native_backend/` implementará:

1. **Tests de Correctness**: Validación de que el código generado produce resultados correctos
2. **Tests de Performance**: Benchmarks comparativos entre diferentes niveles de optimización
3. **Tests de Edge Cases**: Manejo de casos límite y errores
4. **Tests de Integración**: Pipeline completo desde código Vela hasta ejecución nativa
5. **Tests de Cross-platform**: Validación en diferentes plataformas (Windows, Linux, macOS)

### Tipos de Tests Implementados

#### 1. Tests de Correctness (`tests/native_backend/correctness.rs`)
- **Aritmética básica**: Suma, resta, multiplicación, división con tipos int/float
- **Control flow**: If/else, loops, funciones recursivas
- **Arrays y objetos**: Creación, acceso, modificación
- **Funciones**: Llamadas, argumentos, retorno de valores
- **Runtime operations**: GC, signals, actors

#### 2. Tests de Performance (`tests/native_backend/performance.rs`)
- **Benchmarking**: Comparación de rendimiento entre niveles de optimización
- **Memory usage**: Validación de uso de memoria con GC
- **Execution time**: Medición de tiempo de ejecución
- **Scalability**: Tests con diferentes tamaños de input

#### 3. Tests de Edge Cases (`tests/native_backend/edge_cases.rs`)
- **Stack overflow**: Funciones recursivas profundas
- **Memory limits**: Asignación de grandes cantidades de memoria
- **Type limits**: Valores límite de tipos numéricos
- **Error handling**: Manejo de excepciones y errores runtime

#### 4. Tests de Integración (`tests/native_backend/integration.rs`)
- **End-to-end**: Código Vela → IR → LLVM → Ejecutable → Resultado
- **Linking validation**: Verificación de que los ejecutables se linkean correctamente
- **Runtime integration**: Tests con runtime library completa
- **Cross-platform builds**: Generación de ejecutables en diferentes plataformas

### API de Testing

```rust
pub struct NativeBackendTester {
    temp_dir: PathBuf,
    llvm_context: Context,
}

impl NativeBackendTester {
    pub fn new() -> Result<Self, String>
    pub fn compile_and_run(&self, vela_code: &str, opt_level: OptimizationLevel) -> Result<TestResult, String>
    pub fn benchmark_code(&self, vela_code: &str, iterations: usize) -> BenchmarkResult
    pub fn validate_output(&self, expected: &str, actual: &str) -> bool
}

pub struct TestResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
    pub memory_usage: usize,
}

pub struct BenchmarkResult {
    pub o0_time: Duration,
    pub o1_time: Duration,
    pub o2_time: Duration,
    pub o3_time: Duration,
    pub speedup_o1: f64,
    pub speedup_o2: f64,
    pub speedup_o3: f64,
}
```

### Casos de Test Específicos

#### Tests de Correctness
```rust
#[test]
fn test_arithmetic_operations() {
    let vela_code = r#"
        fn main() -> void {
            let a = 10;
            let b = 20;
            let c = a + b * 2;  // 50
            print(c);
        }
    "#;

    let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)?;
    assert_eq!(result.stdout.trim(), "50");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_array_operations() {
    let vela_code = r#"
        fn main() -> void {
            let arr = [1, 2, 3, 4, 5];
            let sum = 0;
            for i in 0..arr.length() {
                sum = sum + arr[i];
            }
            print(sum);  // 15
        }
    "#;

    let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)?;
    assert_eq!(result.stdout.trim(), "15");
}
```

#### Tests de Performance
```rust
#[test]
fn test_optimization_levels_performance() {
    let vela_code = r#"
        fn fibonacci(n: int) -> int {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        fn main() -> void {
            let result = fibonacci(35);
            print(result);
        }
    "#;

    let benchmark = tester.benchmark_code(vela_code, 10)?;

    // O3 debería ser al menos 2x más rápido que O0
    assert!(benchmark.speedup_o3 > 2.0);
    // O2 debería ser más rápido que O1
    assert!(benchmark.speedup_o2 > benchmark.speedup_o1);
}
```

#### Tests de Runtime Features
```rust
#[test]
fn test_garbage_collection() {
    let vela_code = r#"
        fn create_objects(count: int) -> void {
            for i in 0..count {
                let obj = { value: i };
                // obj se vuelve unreachable aquí
            }
        }

        fn main() -> void {
            create_objects(1000);
            print("GC test completed");
        }
    "#;

    let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)?;
    assert_eq!(result.stdout.trim(), "GC test completed");
    assert_eq!(result.exit_code, 0);
    // Verificar que no hubo leaks de memoria
    assert!(result.memory_usage < 10 * 1024 * 1024); // < 10MB
}
```

### Configuración de Tests

#### Cargo.toml Configuration
```toml
[dev-dependencies]
criterion = "0.5"
tempfile = "3.0"
assert_cmd = "2.0"

[[bench]]
name = "native_backend_benchmarks"
harness = false
```

#### Test Organization
```
tests/native_backend/
├── mod.rs                    # Módulo principal
├── correctness.rs           # Tests de correctness
├── performance.rs           # Benchmarks y performance
├── edge_cases.rs            # Casos límite
├── integration.rs           # Tests end-to-end
├── utils.rs                 # Utilidades de testing
└── fixtures/                # Código Vela de prueba
    ├── arithmetic.vela
    ├── control_flow.vela
    ├── arrays.vela
    └── runtime.vela
```

### Métricas de Calidad

#### Coverage Requirements
- **Line coverage**: >= 90% del código del backend nativo
- **Branch coverage**: >= 85% de todas las ramas condicionales
- **Function coverage**: 100% de funciones públicas

#### Performance Baselines
- **Compilation time**: < 5 segundos para programas típicos
- **Execution overhead**: < 10% vs código C equivalente
- **Memory usage**: < 2x del uso de memoria en bytecode VM

## ✅ Criterios de Aceptación
- [x] Suite completa de tests de correctness implementada
- [x] Tests de performance con benchmarks comparativos
- [x] Tests de edge cases para casos límite
- [x] Tests de integración end-to-end
- [x] Cobertura de código >= 90%
- [x] Tests ejecutándose en CI/CD
- [x] Documentación completa de tests
- [x] ADR de testing strategy creado

## 🔗 Referencias
- **Jira:** [TASK-126](https://velalang.atlassian.net/browse/TASK-126)
- **Historia:** [VELA-1123](https://velalang.atlassian.net/browse/VELA-1123)
- **Dependencias:** TASK-121, TASK-122, TASK-123, TASK-124, TASK-125 completadas