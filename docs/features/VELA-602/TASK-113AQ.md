# TASK-113AQ: Diseñar arquitectura de observability

## 📋 Información General
- **Historia:** US-24H: Como desarrollador, quiero observability para monitorear microservicios
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Diseñar una arquitectura completa de observability que incluya distributed tracing, metrics y structured logging para microservicios en Vela.

## 🔨 Implementación

### Arquitectura Diseñada

#### 1. **Distributed Tracing**
- **Framework**: OpenTelemetry como estándar
- **Propagación**: Headers W3C Trace Context
- **Spans jerárquicos**: Contexto automático entre servicios
- **Sampling**: Configurable (always_on, probabilistic, rate-limiting)

#### 2. **Metrics System**
- **Tipos soportados**: Counter, Gauge, Histogram, Summary
- **Backend primario**: Prometheus
- **Etiquetas**: Para segmentación dimensional
- **Agregación**: Cliente-side con finalización server-side

#### 3. **Structured Logging**
- **Formato**: JSON con campos estándar
- **Niveles**: TRACE, DEBUG, INFO, WARN, ERROR
- **Contexto**: Inyección automática de trace_id y span_id
- **Sinks**: Múltiples destinos (consola, archivos, servicios externos)

### Componentes Técnicos

#### Tracing Layer
```rust
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
pub struct MetricsRegistry {
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}
```

#### Logging Layer
```rust
pub struct Logger {
    level: Level,
    sinks: Vec<Box<dyn LogSink>>,
    context: SpanContext,
}
```

### Decorators Planeados

#### @traced
```vela
@traced(name="http_request", attributes={"method": "GET"})
async fn getUsers() -> Result<List<User>> {
    // Automáticamente crea spans y propaga contexto
}
```

#### @metered
```vela
@metered(name="requests_total", labels={"method", "status"})
async fn handleRequest(req: Request) -> Response {
    // Métricas automáticas de latencia y conteo
}
```

#### @logged
```vela
@logged(level="INFO", message="Processing request")
fn processData(data: Data) -> Result<ProcessedData> {
    // Logs con contexto de trace
}
```

## ✅ Criterios de Aceptación
- [x] Arquitectura de tres pilares definida (tracing, metrics, logging)
- [x] Estándares del mercado identificados (OpenTelemetry, Prometheus)
- [x] Componentes técnicos diseñados
- [x] Decorators planeados con ejemplos
- [x] ADR creado en `docs/architecture/`
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [VELA-602](https://velalang.atlassian.net/browse/VELA-602)
- **ADR:** `docs/architecture/ADR-113AQ-001-observability-architecture.md`
- **Historia:** US-24H