# TASK-113AK: Implementar @circuitBreaker decorator

## 📋 Información General
- **Historia:** VELA-601
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar el decorador @circuitBreaker que protege contra fallos en cascada en sistemas distribuidos, siguiendo la arquitectura definida en ADR-113AJ-001.

## 🔨 Implementación

### Arquitectura Implementada
Se implementó el patrón Circuit Breaker con tres estados:

1. **CLOSED**: Estado normal, permite todas las llamadas
2. **OPEN**: Estado de fallo, rechaza todas las llamadas
3. **HALF_OPEN**: Estado de prueba, permite llamadas limitadas para verificar recuperación

### Componentes Creados

#### 1. CircuitBreakerConfig (Rust)
Configuración del comportamiento del circuit breaker:
- `failure_threshold`: Número de fallos para abrir el circuito (default: 5)
- `recovery_timeout`: Segundos para intentar recuperación (default: 30.0)
- `success_threshold`: Éxitos necesarios para cerrar (default: 2)
- `call_timeout`: Timeout por llamada individual (default: 10.0)

#### 2. CircuitBreaker Struct (Rust)
Implementación principal del patrón:
- Gestión de estados (closed/open/half-open)
- Contadores de éxito/fallo
- Lógica de transición entre estados
- Ejecución con timeout usando Tokio

#### 3. Función Helper with_circuit_breaker (Rust)
- `with_circuit_breaker()`: Función para aplicar circuit breaker a cualquier async function
- Gestión de instancias compartidas por nombre
- Integración con runtime de Vela

### Código Generado por el Compilador
Cuando se usa `@circuitBreaker` en Vela:
```vela
@circuitBreaker(failureThreshold=3, recoveryTimeout=10000, successThreshold=2)
async fn callExternalAPI(request: Request) -> Result<Response, Error> {
    // Código original
}
```

Genera código Rust equivalente:
```rust
async fn callExternalAPI(request: Request) -> Result<Response, Error> {
    let cb_config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout: Duration::from_millis(10000),
        success_threshold: 2,
    };

    vela_runtime::resilience::with_circuit_breaker(cb_config, || async {
        // Código original aquí
    }).await
}
```

### Estados y Transiciones

```
CLOSED ──(failure_threshold fallos)──> OPEN
   ▲                                      │
   │                                      │
   └─(recovery_timeout + success_threshold)┘
      HALF_OPEN ←──(1 fallo)───
```

### Manejo de Errores
- **CircuitBreakerOpen**: Error cuando el circuito está abierto
- **Timeout**: Cuando una llamada excede el timeout individual
- **FunctionError**: Errores propagados desde la función protegida

### Tests Implementados
- ✅ Estados y transiciones del circuit breaker
- ✅ Recuperación automática después del timeout
- ✅ Rechazo de llamadas cuando está abierto
- ✅ Timeout por llamada individual
- ✅ Gestión de instancias compartidas

## ✅ Criterios de Aceptación
- [x] CircuitBreaker con 3 estados implementado en Rust
- [x] Configuración completa (thresholds, timeouts)
- [x] Transiciones de estado correctas
- [x] Manejo de timeout por llamada
- [x] Gestión de instancias compartidas
- [x] Tests unitarios con cobertura > 80% (6 tests pasando)
- [x] Integración con runtime de Vela
- [x] Compilación exitosa sin errores

## 🔗 Referencias
- **Jira:** [TASK-113AK](https://velalang.atlassian.net/browse/TASK-113AK)
- **Historia:** [VELA-601](https://velalang.atlassian.net/browse/VELA-601)
- **ADR:** ADR-113AJ-001-resilience-patterns-architecture.md
- **Patrón:** https://docs.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker