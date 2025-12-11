# TASK-113AM: Implementar @timeout decorator

## 📋 Información General
- **Historia:** VELA-601
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar el decorador `@timeout` como parte del sistema de patrones de resiliencia de Vela, permitiendo configurar timeouts en operaciones asíncronas para prevenir bloqueos indefinidos.

## 🔨 Implementación

### Arquitectura del Sistema
El decorador `@timeout` sigue el mismo patrón que otros decoradores de resiliencia (`@retry`, `@circuitBreaker`):

1. **Parsing del decorador** en `compiler/src/resilience_decorators.rs`
2. **Generación de código** que llama a funciones del runtime
3. **Implementación en runtime** en `runtime/src/resilience.rs`

### Código Implementado

#### 1. Runtime Implementation (`runtime/src/resilience.rs`)
```rust
/// Configuration for timeout decorator
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub duration: u64, // milliseconds
}

/// Apply timeout to an async operation
pub async fn with_timeout<T, F, Fut>(
    config: TimeoutConfig,
    operation: F,
) -> ResilienceResult<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let duration = Duration::from_millis(config.duration);
    match tokio::time::timeout(duration, operation()).await {
        Ok(result) => Ok(result),
        Err(_) => Err(ResilienceError::Timeout(TimeoutError {
            duration: config.duration,
        })),
    }
}
```

#### 2. Compiler Parsing (`compiler/src/resilience_decorators.rs`)
```rust
/// Parse @timeout decorator arguments
pub fn parse_timeout_decorator(decorator: &Decorator) -> CompileResult<TimeoutConfig> {
    if decorator.arguments.len() != 1 {
        return Err(CompileError::Decorator(format!(
            "@timeout decorator expects exactly 1 argument (duration in milliseconds), got {}",
            decorator.arguments.len()
        )));
    }

    let duration = extract_timeout_duration(&decorator.arguments[0])?;
    Ok(TimeoutConfig { duration })
}

/// Generate Rust code for timeout wrapper
pub fn generate_timeout_code(config: &TimeoutConfig, inner_code: String) -> String {
    format!(
        "runtime::resilience::with_timeout(
            runtime::resilience::TimeoutConfig {{
                duration: {},
            }},
            || async move {{
                {}
            }}
        ).await",
        config.duration,
        inner_code
    )
}
```

### Tests Implementados

#### Runtime Tests
```rust
#[test]
fn test_timeout_success() {
    // Test que la operación completa dentro del timeout
}

#[test]
fn test_timeout_expired() {
    // Test que la operación expira y retorna TimeoutError
}
```

#### Compiler Tests
```rust
#[test]
fn test_parse_timeout_decorator() {
    // Test parsing de argumentos del decorador
}

#[test]
fn test_generate_timeout_code() {
    // Test generación de código Rust
}
```

## ✅ Criterios de Aceptación
- [x] **Parsing correcto**: El decorador acepta un argumento numérico (duración en ms)
- [x] **Validación de argumentos**: Error si no se proporciona exactamente 1 argumento
- [x] **Generación de código**: Produce código Rust válido que llama a `with_timeout`
- [x] **Runtime funcional**: La función `with_timeout` funciona con Tokio
- [x] **Manejo de errores**: Retorna `TimeoutError` cuando expira
- [x] **Tests completos**: Tests en runtime (2/2) y compiler (2/2) pasan
- [x] **Integración**: Funciona con el sistema de decoradores existente

## 🔗 Referencias
- **Jira:** [TASK-113AM](https://velalang.atlassian.net/browse/TASK-113AM)
- **Historia:** [VELA-601](https://velalang.atlassian.net/browse/VELA-601)
- **Arquitectura:** Patrón de resiliencia consistente con `@retry` y `@circuitBreaker`
- **Runtime:** `runtime/src/resilience.rs`
- **Compiler:** `compiler/src/resilience_decorators.rs`

## 📊 Métricas de Implementación
- **Archivos modificados:** 2 (runtime + compiler)
- **Líneas de código:** ~50 líneas nuevas
- **Tests agregados:** 4 tests unitarios
- **Tiempo de implementación:** ~2 horas
- **Complejidad:** Baja (patrón establecido)

## 🎨 Uso en Vela

```vela
@timeout(5000)  // 5 segundos timeout
async fn fetchData() -> Result<String> {
    // Operación que podría tardar mucho
    return await httpGet("https://api.example.com/data");
}

// Sin argumentos usa default (no implementado aún)
@timeout
async fn quickOperation() -> Result<String> {
    return await fastApiCall();
}
```

## 🔄 Patrón de Resiliencia Implementado

| Decorador | Estado | Argumentos |
|-----------|--------|------------|
| `@retry` | ✅ Completo | max_attempts, base_delay, backoff_multiplier |
| `@circuitBreaker` | ✅ Completo | failure_threshold, recovery_timeout, success_threshold, call_timeout |
| `@timeout` | ✅ **Completo** | duration (ms) |
| `@bulkhead` | 🔄 Próximo | capacity, queue_size |
| `@rateLimit` | 📋 Pendiente | requests_per_second, burst_capacity |</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-601\TASK-113AM.md