# ADR-113AQ-001: Arquitectura de Observability para Vela

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
Como parte del EPIC-09H: Microservices - Observability, necesitamos implementar un sistema completo de observability para monitorear microservicios en Vela. La observability incluye tres pilares fundamentales:

1. **Distributed Tracing**: Seguimiento de requests a través de múltiples servicios
2. **Metrics**: Métricas cuantitativas sobre el rendimiento del sistema
3. **Structured Logging**: Logs estructurados para debugging y análisis

El sistema debe ser:
- **Type-safe**: Integración nativa con el sistema de tipos de Vela
- **Decorator-driven**: Uso de decoradores para configuración declarativa
- **Performance-aware**: Bajo overhead en producción
- **Standards-compliant**: Compatible con OpenTelemetry, Prometheus, etc.

## Decisión
Implementaremos una arquitectura de observability basada en tres componentes principales:

### 1. Sistema de Tracing Distribuido
- **Protocolo**: OpenTelemetry como estándar
- **Propagación**: W3C Trace Context headers
- **Spans**: Jerarquía de operaciones con contexto
- **Sampling**: Configurable (always_on, probabilistic, rate-limiting)

### 2. Sistema de Métricas
- **Tipos**: Counter, Gauge, Histogram, Summary
- **Backend**: Prometheus como estándar primario
- **Etiquetas**: Dimensiones para segmentación de métricas
- **Agregación**: Cliente-side aggregation con server-side finalización

### 3. Sistema de Logging Estructurado
- **Formato**: JSON estructurado con campos estándar
- **Niveles**: TRACE, DEBUG, INFO, WARN, ERROR
- **Contexto**: Inyección automática de trace_id, span_id
- **Sinks**: Consola, archivos, servicios externos

## Consecuencias

### Positivas
- **Visibilidad completa**: Tracing end-to-end de requests
- **Monitoreo proactivo**: Métricas para alerting y dashboards
- **Debugging eficiente**: Logs correlacionados con traces
- **Estandarización**: Compatible con herramientas del ecosistema
- **Bajo overhead**: Sampling y buffering para performance

### Negativas
- **Complejidad**: Tres sistemas distintos a mantener
- **Dependencias**: Nuevas dependencias de crates de observability
- **Configuración**: Setup inicial más complejo para desarrolladores

## Alternativas Consideradas

### 1. Sistema Unificado (Rechazado)
**Alternativa**: Un solo sistema que combine tracing, metrics y logging
- **Pros**: Simplicidad conceptual
- **Cons**: Menos flexible, no estándares del mercado
- **Razón de rechazo**: Los tres pilares son conceptualmente distintos y tienen estándares separados

### 2. Solo Metrics (Rechazado)
**Alternativa**: Solo implementar métricas, usar logging existente
- **Pros**: Implementación más simple
- **Cons**: Falta tracing distribuido, debugging limitado
- **Razón de rechazo**: Necesitamos observability completa para microservicios

### 3. Vendor Lock-in (Rechazado)
**Alternativa**: Usar solo herramientas de un vendor (ej: DataDog)
- **Pros**: Integración más fácil
- **Cons**: Lock-in, costos elevados
- **Razón de rechazo**: Necesitamos estándares abiertos y flexibilidad

## Implementación

### Arquitectura General
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Vela Service  │    │   Vela Service  │    │   Vela Service  │
│                 │    │                 │    │                 │
│  ┌────────────┐ │    │  ┌────────────┐ │    │  ┌────────────┐ │
│  │ @traced    │ │    │  │ @traced    │ │    │  │ @traced    │ │
│  │ function   │◄┼────┼──┤ function   │◄┼────┼──┤ function   │ │
│  └────────────┘ │    │  └────────────┘ │    │  └────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │     OpenTelemetry        │
                    │    Collector/Agent       │
                    └───────────────────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    │             │             │
            ┌─────────────┐ ┌──────────┐ ┌─────────────┐
            │  Jaeger     │ │Prometheus│ │  ELK Stack  │
            │ (Tracing)   │ │(Metrics) │ │ (Logging)   │
            └─────────────┘ └──────────┘ └─────────────┘
```

### Componentes Técnicos

#### Tracing Layer
```rust
// runtime/src/observability/tracing.rs
pub struct Tracer {
    provider: opentelemetry::sdk::trace::TracerProvider,
    sampler: Box<dyn Sampler>,
}

pub struct Span {
    inner: opentelemetry::trace::Span,
    attributes: HashMap<String, Value>,
}
```

#### Metrics Layer
```rust
// runtime/src/observability/metrics.rs
pub struct MetricsRegistry {
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}

pub trait Metric {
    fn record(&self, value: f64, labels: &Labels);
}
```

#### Logging Layer
```rust
// runtime/src/observability/logging.rs
pub struct Logger {
    level: Level,
    sinks: Vec<Box<dyn LogSink>>,
    context: SpanContext,
}

pub trait LogSink {
    fn log(&self, record: &LogRecord) -> Result<(), Error>;
}
```

### Decorators en Vela

#### @traced Decorator
```vela
@traced(name="http_request", attributes={"method": "GET", "path": "/api/users"})
async fn getUsers() -> Result<List<User>> {
    // Esta función será automáticamente traced
    return await userService.findAll()
}
```

#### @metered Decorator
```vela
@metered(name="http_requests_total", labels={"method", "status"})
async fn handleRequest(req: Request) -> Response {
    // Métricas automáticas de latencia y conteo
    return await processRequest(req)
}
```

#### @logged Decorator
```vela
@logged(level="INFO", message="Processing user request")
fn processUser(userId: String) -> Result<User> {
    // Logs estructurados con contexto de trace
    return userRepo.findById(userId)
}
```

## Referencias
- Jira: [VELA-602](https://velalang.atlassian.net/browse/VELA-602)
- OpenTelemetry: https://opentelemetry.io/
- Prometheus: https://prometheus.io/
- Epic: EPIC-09H: Microservices - Observability

## Implementación
Ver código en: `runtime/src/observability/`
Documentación: `docs/features/VELA-602/`