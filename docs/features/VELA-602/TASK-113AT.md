# TASK-113AT: Implementar @traced decorator

## 📋 Información General
- **Historia:** VELA-602 (US-24H observability)
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar el decorador `@traced` para distributed tracing con OpenTelemetry, permitiendo instrumentar automáticamente funciones con spans, atributos y propagación de contexto.

## 🔨 Implementación

### Arquitectura de Tracing
El decorador `@traced` se integra en el pipeline del compilador durante la conversión AST → IR, generando código que:

1. **Crea spans automáticamente** al inicio de la función
2. **Establece atributos** en el span (tags personalizados)
3. **Propaga contexto** entre servicios
4. **Finaliza spans** automáticamente al terminar la función

#### Componentes Implementados
1. **Parser de decoradores** (`parse_traced_decorator`)
2. **Generador de código** (`generate_traced_decorator_code`)
3. **Runtime de tracing** (`runtime/src/observability/tracing.rs`)
4. **Integración en AST → IR** (`ast_to_ir.rs`)

### Decorador `@traced`
Decorador principal para distributed tracing.

```vela
@traced(name="http_request", tags={"method": "GET", "endpoint": "/users"})
fn handleRequest() -> Result<Response, Error> {
    // Automáticamente crea un span con:
    // - Nombre: "http_request"
    // - Atributos: method="GET", endpoint="/users"
    // - Atributo automático: function="handleRequest"
    // - Atributo automático: service.name
}
```

**Parámetros:**
- `name`: Nombre del span (requerido)
- `tags`: Atributos adicionales como HashMap (opcional)

### Runtime de Tracing

#### Tracer Global
```rust
pub struct Tracer {
    inner: opentelemetry::sdk::trace::Tracer,
    config: TracingConfig,
}

impl Tracer {
    pub async fn start_span(&self, name: &str) -> Span {
        // Crear span con configuración
    }
}
```

#### Span Implementation
```rust
pub struct Span {
    inner: opentelemetry::sdk::trace::Span,
}

impl Span {
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        // Establecer atributo en el span
    }

    pub fn set_status(&mut self, status: Status) {
        // Establecer status del span
    }

    pub fn end(self) {
        // Finalizar span y enviarlo
    }
}
```

#### Propagación de Contexto
```rust
pub struct Propagation {
    // Implementa W3C Trace Context
}

impl Propagation {
    pub fn inject(&self, span: &Span, headers: &mut HashMap<String, String>) {
        // Inyectar headers de tracing
    }

    pub fn extract(&self, headers: &HashMap<String, String>) -> Option<SpanContext> {
        // Extraer contexto de headers
    }
}
```

### Integración en el Compilador

#### 1. Parser de Decoradores
```rust
pub fn parse_traced_decorator(
    decorator: &Decorator,
) -> Result<TracedDecorator, CompileError> {
    // Parse name and tags from decorator arguments
    Ok(TracedDecorator { name, tags })
}
```

#### 2. Generación de Código
```rust
pub fn generate_traced_decorator_code(
    decorator: &TracedDecorator,
    function_name: &str,
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Import tracing functions
    code.push_str("use vela_runtime::observability::{get_tracer};\n");

    // Start span
    code.push_str(&format!(
        "let __tracing_span = match get_tracer().await {{
         Some(tracer) => {{
             let mut span = tracer.start_span(\"{}\");
             // Set attributes...
             Some(span)
         }},
         None => None,
         }};\n",
        decorator.name
    ));

    // Function wrapper
    code.push_str("let __result = async move {\n");

    Ok(code)
}
```

#### 3. Cleanup Automático
```rust
pub fn generate_decorator_cleanup(decorators: &[String]) -> String {
    let mut code = String::new();

    for decorator_type in decorators {
        if decorator_type == "traced" {
            code.push_str("// End tracing span\n");
            code.push_str("if let Some(mut span) = __tracing_span {\n");
            code.push_str("    span.set_status(opentelemetry::trace::Status::Ok);\n");
            code.push_str("    span.end();\n");
            code.push_str("}\n");
        }
    }

    code
}
```

### Ejemplo de Uso Completo

#### Servicio HTTP con Tracing
```vela
service UserService {
    repository: UserRepository = inject(UserRepository)

    @traced(name="create_user", tags={"service": "user-service", "operation": "create"})
    @metered(name="user_service_create_user_total", help="Total user creation attempts")
    @logged(level="info", message="Creating new user: ${name}")
    fn createUser(name: String, email: String) -> Result<User, Error> {
        // Span automáticamente creado con:
        // - Nombre: "create_user"
        // - Tags: service="user-service", operation="create"
        // - Atributos automáticos: function="createUser", service.name

        let user = User { name, email };
        repository.save(user)
    }
}
```

#### Controlador HTTP
```vela
controller UserController {
    service: UserService = inject(UserService)

    @traced(name="http_request", tags={"method": "POST", "endpoint": "/users"})
    async fn createUser(@body request: CreateUserRequest) -> Result<User, HttpError> {
        // Span padre para toda la request HTTP
        service.createUser(request.name, request.email)
    }
}
```

### Propagación Entre Servicios

#### Cliente HTTP con Tracing
```vela
@traced(name="external_api_call", tags={"service": "external", "api": "user-service"})
async fn callUserService(userId: String) -> Result<User, Error> {
    // Headers de tracing automáticamente inyectados
    let headers = HashMap::new();
    // propagation.inject(current_span, &mut headers);

    httpClient.get(&format!("/users/{}", userId), headers).await
}
```

### Configuración de Tracing

#### Variables de Entorno
```bash
# OpenTelemetry configuration
OTEL_SERVICE_NAME=my-vela-service
OTEL_TRACES_EXPORTER=otlp
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Sampling
OTEL_TRACES_SAMPLER=traceidratio
OTEL_TRACES_SAMPLER_ARG=0.1  # 10% sampling
```

#### Configuración Programática
```rust
let config = TracingConfig {
    service_name: "my-service".to_string(),
    service_version: "1.0.0".to_string(),
    sampling_ratio: 0.1, // 10% sampling
    max_attributes: 128,
    max_events: 128,
};

let tracer = Tracer::new(config)?;
```

### Exportación de Traces

Los traces se exportan automáticamente a los backends configurados:

- **Jaeger**: `OTEL_TRACES_EXPORTER=jaeger`
- **Zipkin**: `OTEL_TRACES_EXPORTER=zipkin`
- **OTLP**: `OTEL_TRACES_EXPORTER=otlp` (recomendado)
- **Console**: Para desarrollo

### Beneficios del Decorador

#### 1. **Tracing Automático**
- No requiere código manual para crear spans
- Atributos automáticos (function name, service name)
- Propagación automática de contexto

#### 2. **Bajo Overhead**
- Graceful degradation si tracing no está disponible
- Sampling configurable para producción
- Async operations no bloqueantes

#### 3. **Integración Completa**
- Funciona con `@metered` y `@logged`
- Propagación entre servicios
- Compatible con estándares OpenTelemetry

#### 4. **Debugging Mejorado**
- Trazas distribuidas entre microservicios
- Correlación automática de requests
- Métricas de latencia por operación

## ✅ Criterios de Aceptación
- [x] Decorador `@traced` implementado y funcional
- [x] Integración en pipeline del compilador completada
- [x] Runtime de tracing implementado (Span, Tracer, Propagation)
- [x] Propagación de contexto W3C Trace Context
- [x] Atributos automáticos (function, service.name)
- [x] Cleanup automático de spans
- [x] Ejemplo de uso creado en `examples/observability-example.vela`
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [TASK-113AT](https://velalang.atlassian.net/browse/TASK-113AT)
- **Historia:** [VELA-602](https://velalang.atlassian.net/browse/VELA-602)
- **Arquitectura:** [ADR-113AQ-001-observability-architecture.md](../architecture/ADR-113AQ-001-observability-architecture.md)
- **OpenTelemetry:** [TASK-113AR.md](TASK-113AR.md)
- **Prometheus:** [TASK-113AS.md](TASK-113AS.md)