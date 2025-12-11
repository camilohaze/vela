# ADR-113AJ-001: Arquitectura de Resilience Patterns

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
VELA-600 implementó patrones de resiliencia específicos para message brokers (retry, circuit breaker, DLQ). Sin embargo, los microservicios necesitan patrones de resiliencia más generales que puedan aplicarse a cualquier operación asíncrona, llamada HTTP, acceso a base de datos, o integración externa.

Esta ADR define la arquitectura para implementar decoradores de resiliencia generales que puedan aplicarse a cualquier función o método en Vela, siguiendo el patrón de decoradores declarativos establecido en VELA-600.

## Decisión
Implementaremos un sistema de decoradores de resiliencia que se integre con el compilador de Vela, generando código automático para aplicar patrones de resiliencia en tiempo de compilación.

### Arquitectura General
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Decoradores   │ -> │   Generador de   │ -> │   Código Rust   │
│   @retry        │    │   Código AST     │    │   con Tokio     │
│   @circuitBreaker│   │                  │    │                 │
│   @timeout      │    │   Transforma     │    │   Resilience    │
│   @bulkhead     │    │   funciones      │    │   Patterns      │
│   @fallback     │    │   decoradas      │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Decoradores a Implementar

#### 1. @retry
```vela
@retry(maxAttempts=3, backoff="exponential", baseDelay=1000)
async fn processPayment(payment: Payment) -> Result<PaymentResult, Error> {
    // Código que puede fallar
}
```

**Genera:**
```rust
async fn processPayment(payment: Payment) -> Result<PaymentResult, Error> {
    let retry_config = RetryConfig {
        max_attempts: 3,
        backoff_strategy: BackoffStrategy::Exponential,
        base_delay: Duration::from_millis(1000),
    };

    vela_runtime::resilience::with_retry(retry_config, || async {
        // Código original
    }).await
}
```

#### 2. @circuitBreaker
```vela
@circuitBreaker(failureThreshold=5, recoveryTimeout=30000, successThreshold=2)
async fn callExternalAPI(request: Request) -> Result<Response, Error> {
    // Llamada a API externa
}
```

**Genera:**
```rust
async fn callExternalAPI(request: Request) -> Result<Response, Error> {
    let cb_config = CircuitBreakerConfig {
        failure_threshold: 5,
        recovery_timeout: Duration::from_millis(30000),
        success_threshold: 2,
    };

    vela_runtime::resilience::with_circuit_breaker(cb_config, || async {
        // Código original
    }).await
}
```

#### 3. @timeout
```vela
@timeout(duration=5000, onTimeout="throw")
async fn slowOperation(data: Data) -> Result<Result, Error> {
    // Operación que puede ser lenta
}
```

**Genera:**
```rust
async fn slowOperation(data: Data) -> Result<Result, Error> {
    let timeout_config = TimeoutConfig {
        duration: Duration::from_millis(5000),
        on_timeout: TimeoutAction::Throw,
    };

    vela_runtime::resilience::with_timeout(timeout_config, || async {
        // Código original
    }).await
}
```

#### 4. @bulkhead
```vela
@bulkhead(maxConcurrent=10, queueSize=50)
async fn databaseOperation(query: Query) -> Result<Rows, Error> {
    // Operación de base de datos
}
```

**Genera:**
```rust
async fn databaseOperation(query: Query) -> Result<Rows, Error> {
    let bulkhead_config = BulkheadConfig {
        max_concurrent: 10,
        queue_size: 50,
    };

    vela_runtime::resilience::with_bulkhead(bulkhead_config, || async {
        // Código original
    }).await
}
```

#### 5. @fallback
```vela
@fallback(fallbackFn="getDefaultUser", exceptions=[NetworkError, TimeoutError])
async fn getUserFromCache(userId: String) -> Result<User, Error> {
    // Intentar obtener de cache
}

async fn getDefaultUser(userId: String) -> Result<User, Error> {
    // Fallback: obtener de base de datos
}
```

**Genera:**
```rust
async fn getUserFromCache(userId: String) -> Result<User, Error> {
    let fallback_config = FallbackConfig {
        fallback_fn: "getDefaultUser",
        exceptions: vec![NetworkError::type_id(), TimeoutError::type_id()],
    };

    vela_runtime::resilience::with_fallback(fallback_config, || async {
        // Código original
    }).await
}
```

### Composición de Decoradores
Los decoradores pueden combinarse en orden específico:
```vela
@circuitBreaker(failureThreshold=3, recoveryTimeout=10000)
@retry(maxAttempts=2, backoff="linear", baseDelay=500)
@timeout(duration=2000)
async fn criticalOperation(data: Data) -> Result<Result, Error> {
    // Operación crítica con múltiples capas de resiliencia
}
```

**Orden de aplicación (de afuera hacia adentro):**
1. Circuit Breaker (primero, protege contra fallos masivos)
2. Retry (reintenta operaciones fallidas)
3. Timeout (limita tiempo de ejecución)

### Integración con el Compilador
1. **Parser**: Extender para reconocer decoradores de resiliencia
2. **AST**: Agregar nodos para decoradores de resiliencia
3. **Codegen**: Generar código Rust con llamadas a runtime de resiliencia
4. **Runtime**: Implementar funciones de resiliencia en `vela-runtime`

### Runtime de Resiliencia
Crear crate `vela-runtime` con módulo `resilience`:
```rust
pub mod resilience {
    pub async fn with_retry<F, Fut, T, E>(config: RetryConfig, f: F) -> Result<T, E>
    where F: Fn() -> Fut, Fut: Future<Output = Result<T, E>> { ... }

    pub async fn with_circuit_breaker<F, Fut, T, E>(config: CircuitBreakerConfig, f: F) -> Result<T, E>
    where F: Fn() -> Fut, Fut: Future<Output = Result<T, E>> { ... }

    pub async fn with_timeout<F, Fut, T>(config: TimeoutConfig, f: F) -> Result<T, TimeoutError>
    where F: Fn() -> Fut, Fut: Future<Output = T> { ... }

    pub async fn with_bulkhead<F, Fut, T, E>(config: BulkheadConfig, f: F) -> Result<T, E>
    where F: Fn() -> Fut, Fut: Future<Output = Result<T, E>> { ... }

    pub async fn with_fallback<F, Fut, T, E, Fb, FbFut>(
        config: FallbackConfig, f: F, fallback: Fb
    ) -> Result<T, E>
    where F: Fn() -> Fut, Fut: Future<Output = Result<T, E>>,
          Fb: Fn() -> FbFut, FbFut: Future<Output = Result<T, E>> { ... }
}
```

## Consecuencias

### Positivas
- **Resiliencia declarativa**: Los desarrolladores pueden agregar resiliencia con simples decoradores
- **Composición flexible**: Múltiples patrones pueden combinarse según necesidades
- **Performance**: Código generado en compile-time, sin runtime overhead de reflexión
- **Type safety**: Verificación de tipos en tiempo de compilación
- **Testability**: Cada patrón puede testearse independientemente

### Negativas
- **Complejidad del compilador**: Aumenta significativamente la complejidad del codegen
- **Runtime crate**: Dependencia adicional para aplicaciones Vela
- **Debugging**: El código generado puede ser más difícil de debuggear
- **Memory overhead**: Algunos patrones requieren estado adicional

### Trade-offs
- **Vs implementación manual**: Menos control fino pero más productividad
- **Vs runtime-only**: Mejor performance pero mayor complejidad de implementación
- **Vs librerías externas**: Mejor integración con Vela pero acoplamiento al runtime

## Alternativas Consideradas

### 1. Runtime-only con reflexión
**Rechazada porque:**
- Performance overhead significativo
- No type safety en configuración
- Debugging más difícil
- No se alinea con filosofía de Vela (compile-time optimizations)

### 2. Macros procedurales en lugar de decoradores
**Rechazada porque:**
- Menos legible que decoradores
- No reutilizable entre funciones
- Mayor verbosidad

### 3. Librería externa en lugar de runtime integrado
**Rechazada porque:**
- No integración nativa con async/await de Vela
- Configuración más compleja
- Menos optimización para el runtime de Vela

## Implementación
Ver código en:
- `compiler/src/resilience_decorators.rs` - Parser y codegen
- `runtime/src/resilience.rs` - Implementación de patrones
- `docs/features/VELA-601/TASK-113AJ.md` - Documentación detallada

## Referencias
- Jira: VELA-601
- ADR previa: ADR-113AG-001-decoradores-consumer-subscribe.md
- Patrón: https://docs.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker
- Patrón: https://docs.microsoft.com/en-us/azure/architecture/patterns/retry
- Patrón: https://docs.microsoft.com/en-us/azure/architecture/patterns/bulkhead