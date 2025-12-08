# TASK-113P: Implementar log filtering y sampling

## 📋 Información General
- **Historia:** VELA-597 (US-24C)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-08

## 🎯 Objetivo
Implementar sistema avanzado de filtering y sampling para controlar qué logs se procesan y reducir volumen de logs en producción.

## 🔨 Implementación

### Sistema de Filtros Avanzados

#### Tipos de Filtros Implementados

##### 1. Filtros Personalizados
```rust
let config = LogConfig::default()
    .with_filter(|record: &LogRecord| {
        // Filtrar logs que contengan errores de validación
        !record.message.contains("validation error")
    });
```

##### 2. Filtros por Metadata (Exclusión)
```rust
let config = LogConfig::default()
    .exclude_by_metadata("component", "test")
    .exclude_by_metadata("environment", "development");
```

##### 3. Filtros por Metadata (Inclusión Exclusiva)
```rust
let config = LogConfig::default()
    .include_only_by_metadata("service", "critical-service");
```

### Sampling para Reducción de Volumen

#### Configuración de Sampling Rate
```rust
// Solo procesar 10% de los logs
let config = LogConfig::production()
    .with_sampling_rate(0.1);

// Procesar todos los logs (desarrollo)
let config = LogConfig::development()
    .with_sampling_rate(1.0);
```

#### Algoritmo de Sampling
- **Hash-based sampling**: Usa hash consistente del timestamp + mensaje
- **Determinístico**: Mismo log siempre se incluye/excluye
- **Configurable**: Rate de 0.0 (ninguno) a 1.0 (todos)

### Rate Limiting

#### Configuración de Rate Limiting
```rust
// Máximo 100 logs por segundo
let config = LogConfig::production()
    .with_rate_limit(100);

// Sin límite (desarrollo)
let config = LogConfig::development()
    .with_rate_limit(-1);
```

#### Implementación Técnica
- **Ventana deslizante**: Reset cada segundo
- **Thread-safe**: Usa AtomicI64 para contador
- **Compartido**: Estado compartido entre instancias de Logger

### Integración con Logger

#### Filtros Aplicados Automáticamente
```rust
let config = LogConfig::default()
    .with_level(Level::INFO)
    .with_sampling_rate(0.5)
    .exclude_by_metadata("level", "trace");

let logger = Logger::new("app", Arc::new(config), ());

// Todos los filtros se aplican automáticamente
logger.info("This message will be filtered").await?;
```

#### Orden de Aplicación de Filtros
1. **Nivel mínimo**: Verificación básica de level
2. **Filtros personalizados**: Closures definidas por usuario
3. **Sampling**: Reducción probabilística de volumen
4. **Rate limiting**: Límite de frecuencia por segundo

### Configuraciones Predefinidas

#### Configuración de Desarrollo
```rust
LogConfig::development()
// level: DEBUG
// sampling_rate: 1.0 (todos los logs)
// rate_limit: -1 (sin límite)
// thread_id: true
```

#### Configuración de Producción
```rust
LogConfig::production()
// level: WARN
// sampling_rate: 0.1 (10% de logs)
// rate_limit: 100 logs/segundo
// structured: true
```

### Estado Compartido para Rate Limiting

#### RateLimitState
```rust
struct RateLimitState {
    last_reset: Mutex<Instant>,
    counter: AtomicI64,
}
```

**Características:**
- **Compartido**: Arc<RateLimitState> entre múltiples LogConfig
- **Thread-safe**: Mutex para timestamp, Atomic para contador
- **Preciso**: Reset automático cada segundo

## ✅ Criterios de Aceptación
- [x] Filtros personalizados con closures implementados
- [x] Filtros por metadata (include/exclude) funcionando
- [x] Sampling rate configurable implementado
- [x] Rate limiting por segundo implementado
- [x] Estado compartido thread-safe implementado
- [x] Integración automática con Logger funcionando
- [x] Tests de filtering y sampling pasando
- [x] Configuraciones predefinidas (dev/prod) actualizadas

## 🔗 Referencias
- **Jira:** [TASK-113P](https://velalang.atlassian.net/browse/TASK-113P)
- **Historia:** [VELA-597](https://velalang.atlassian.net/browse/VELA-597)
- **Dependencias:** TASK-113M (Logger class)