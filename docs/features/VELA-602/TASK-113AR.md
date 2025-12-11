# TASK-113AR: Implementar OpenTelemetry integration

## 📋 Información General
- **Historia:** US-24H: Como desarrollador, quiero observability para monitorear microservicios
- **Estado:** Completada ✅
- **Fecha:** 2025-12-11

## 🎯 Objetivo
Implementar integración completa con OpenTelemetry para distributed tracing en microservicios Vela.

## 🔨 Implementación

### Componentes Implementados

#### 1. **Tracing Module** (`runtime/src/observability/tracing.rs`)
- **Tracer**: Configuración y creación de tracers OpenTelemetry
- **Span**: Representación de operaciones individuales con atributos
- **SpanContext**: Contexto de trace para propagación entre servicios
- **Propagation**: Utilidades para inyectar/extraer headers W3C Trace Context

#### 2. **Metrics Module** (`runtime/src/observability/metrics.rs`)
- **Counter**: Métricas que solo incrementan (requests totales)
- **Gauge**: Métricas que suben y bajan (conexiones activas)
- **Histogram**: Distribuciones de valores (latencias)
- **Summary**: Cuantiles sobre ventanas de tiempo
- **Prometheus Export**: Formato nativo de Prometheus

#### 3. **Logging Module** (`runtime/src/observability/logging.rs`)
- **LogRecord**: Estructura JSON con campos estándar
- **LogSink**: Trait para múltiples destinos (consola, archivos, memoria)
- **Logger**: Instancia principal con configuración de nivel y sinks
- **Trace Context**: Inyección automática de trace_id y span_id

#### 4. **Módulo Principal** (`runtime/src/observability/mod.rs`)
- **ObservabilityConfig**: Configuración unificada del sistema
- **init_observability()**: Inicialización completa del sistema
- **shutdown_observability()**: Apagado ordenado

### Dependencias Agregadas
```toml
# OpenTelemetry ecosystem
opentelemetry = { version = "0.21", features = ["trace", "metrics"] }
opentelemetry-sdk = { version = "0.21", features = ["trace", "metrics", "rt-tokio"] }
opentelemetry-otlp = { version = "0.14", features = ["grpc-sys-tokio", "trace", "metrics"] }

# Utilities
once_cell = "1.19"
chrono = { version = "0.4", features = ["serde"] }
```

### Arquitectura de Tracing

#### Configuración del Tracer
```rust
let config = TracingConfig {
    service_name: "user-service".to_string(),
    service_version: "1.0.0".to_string(),
    sampling_ratio: 1.0, // Sample all in development
    max_attributes: 128,
    max_events: 128,
};

init_tracing(config).await?;
```

#### Creación de Spans
```rust
let tracer = get_tracer().await.unwrap();
let mut span = tracer.start_span("process_user_request");

// Agregar atributos
span.set_attribute("user_id", opentelemetry::Value::String("123".to_string()));
span.set_attribute("method", opentelemetry::Value::String("POST".to_string()));

// Agregar eventos
span.add_event("validation_started", vec![]);
span.add_event("database_query", vec![]);

// Finalizar span
span.end();
```

#### Propagación de Contexto
```rust
// Extraer contexto de headers HTTP
let headers = HashMap::from([
    ("traceparent".to_string(), "00-12345678901234567890123456789012-1234567890123456-01".to_string()),
]);
let context = get_propagation().extract(&headers);

// Inyectar contexto en headers de respuesta
let mut response_headers = HashMap::new();
if let Some(ctx) = context {
    get_propagation().inject(&ctx, &mut response_headers);
}
```

### Sistema de Métricas

#### Registro de Métricas
```rust
let registry = get_metrics().await.unwrap();

// Registrar métricas
registry.register_counter("http_requests_total", "Total HTTP requests").await?;
registry.register_histogram("http_request_duration", "HTTP request duration").await?;
registry.register_gauge("active_connections", "Active connections").await?;
```

#### Uso de Métricas
```rust
// Counter
let counter = registry.get_counter("http_requests_total").unwrap();
counter.increment().await;

// Histogram
let histogram = registry.get_histogram("http_request_duration").unwrap();
histogram.observe(0.145).await; // 145ms

// Gauge
let gauge = registry.get_gauge("active_connections").unwrap();
gauge.set(42.0).await;
```

#### Export a Prometheus
```rust
let prometheus_output = registry.export_prometheus();
// Output:
// # HELP http_requests_total Total HTTP requests
// # TYPE http_requests_total counter
// http_requests_total 150
//
// # HELP http_request_duration HTTP request duration
// # TYPE http_request_duration histogram
// http_request_duration_bucket{le="0.005"} 0
// ...
```

### Sistema de Logging

#### Configuración del Logger
```rust
let config = LoggerConfig {
    name: "user-service".to_string(),
    level: Level::INFO,
    include_location: false,
};

let sinks: Vec<Box<dyn LogSink>> = vec![
    Box::new(ConsoleSink::new()),
    Box::new(FileSink::new("app.log").await?),
];

init_logging(config, sinks).await?;
```

#### Logging Estructurado
```rust
let logger = get_logger().await.unwrap();

// Log simple
logger.info("User service started").await?;

// Log con campos estructurados
let fields = HashMap::from([
    ("user_id".to_string(), json!(123)),
    ("action".to_string(), json!("login")),
    ("ip".to_string(), json!("192.168.1.1")),
]);
logger.log_with_fields(Level::INFO, "User login attempt", fields).await?;
```

#### Output JSON
```json
{
  "timestamp": "2025-12-11T10:30:00.123Z",
  "level": "INFO",
  "logger": "user-service",
  "message": "User login attempt",
  "fields": {
    "user_id": 123,
    "action": "login",
    "ip": "192.168.1.1"
  },
  "trace_id": "12345678901234567890123456789012",
  "span_id": "1234567890123456"
}
```

## ✅ Criterios de Aceptación
- [x] OpenTelemetry integration completa implementada
- [x] Sistema de métricas Prometheus-compatible implementado
- [x] Logging estructurado con múltiples sinks implementado
- [x] Módulo observability agregado al runtime
- [x] Dependencias agregadas al Cargo.toml
- [x] Configuración unificada implementada
- [x] Tests básicos incluidos
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [VELA-602](https://velalang.atlassian.net/browse/VELA-602)
- **Arquitectura:** `docs/architecture/ADR-113AQ-001-observability-architecture.md`
- **Código:** `runtime/src/observability/`
- **Historia:** US-24H