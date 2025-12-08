# ADR-113L: Arquitectura del Sistema de Logging Estructurado

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Vela necesita un sistema de logging robusto y estructurado que permita a los desarrolladores debuggear aplicaciones, monitorear comportamiento en producción y mantener observabilidad. El sistema debe ser:

- **Estructurado**: Logs en formato JSON con metadata consistente
- **Jerárquico**: Niveles de logging (DEBUG, INFO, WARN, ERROR, FATAL)
- **Configurable**: Múltiples transports (console, file, HTTP, syslog)
- **Performante**: Bajo overhead en producción
- **Type-safe**: Integración con el sistema de tipos de Vela
- **Extensible**: Fácil agregar nuevos transports y formatters

## Decisión
Implementar un sistema de logging estructurado con arquitectura de tres capas:

### 1. Logger Core (Logger<T>)
- Clase genérica `Logger<T>` donde T es el contexto (módulo/componente)
- Métodos: `debug()`, `info()`, `warn()`, `error()`, `fatal()`
- Metadata automática: timestamp, level, context, thread_id
- Lazy evaluation de mensajes para performance

### 2. Transports Layer
- Interface `LogTransport` para diferentes destinos
- Transports built-in: Console, File, HTTP, Syslog
- Configuración por transport: level filtering, formatting
- Async writing para no bloquear aplicación

### 3. Structured Data
- Logs en formato JSON con campos estandarizados
- Campos: `timestamp`, `level`, `message`, `context`, `metadata`, `error`
- Metadata extensible con `HashMap<String, Value>`
- Correlation IDs para tracing distribuido

## Consecuencias

### Positivas
- **Observabilidad**: Logs estructurados facilitan análisis y alerting
- **Performance**: Lazy evaluation y async writing minimizan impacto
- **Flexibilidad**: Múltiples transports y configuración granular
- **Type Safety**: Integración con sistema de tipos de Vela
- **Escalabilidad**: Soporte para high-volume logging

### Negativas
- **Complejidad**: Tres capas añaden complejidad inicial
- **Configuración**: Requiere configuración cuidadosa de transports
- **Serialización**: JSON serialization overhead (mitigado con lazy eval)

## Alternativas Consideradas

### 1. Logging Minimalista (Rechazado)
- Solo console output básico
- Sin estructura ni metadata
- **Razón**: No escala para aplicaciones complejas ni producción

### 2. Wrapper de Librería Externa (Rechazado)
- Usar `tracing` o `log` crate directamente
- **Razón**: No se integra con keywords específicos de Vela (`@logger`)

### 3. Logging Procedural (Rechazado)
- Solo funciones globales `log_info()`, `log_error()`
- **Razón**: No permite contextos jerárquicos ni configuración granular

## Implementación

### API de Uso
```rust
// Logger contextual
let logger = Logger::new("UserService");

// Logging estructurado
logger.info("User created", metadata!{
    user_id: user.id,
    email: user.email,
    timestamp: now()
});

// Logging con error
logger.error("Database connection failed", error, metadata!{
    connection_string: db_url,
    retry_count: attempts
});
```

### Configuración
```rust
let config = LogConfig {
    level: Level::INFO,
    transports: vec![
        Box::new(ConsoleTransport::new()),
        Box::new(FileTransport::new("app.log")),
        Box::new(HttpTransport::new(webhook_url)),
    ],
    structured: true,
};
```

### Keywords de Vela
```rust
@logger(config: LogConfig)
service UserService {
    // Logger automáticamente inyectado
    logger: Logger<UserService>

    fn createUser(user: UserDTO) -> Result<User> {
        logger.info("Creating user", metadata!{ email: user.email });

        match self.save(user) {
            Ok(user) => {
                logger.info("User created", metadata!{ user_id: user.id });
                Ok(user)
            }
            Err(error) => {
                logger.error("Failed to create user", error, metadata!{
                    email: user.email
                });
                Err(error)
            }
        }
    }
}
```

## Referencias
- Jira: [VELA-597](https://velalang.atlassian.net/browse/VELA-597)
- Documentación: docs/features/VELA-597/
- Inspiración: Winston (Node.js), Serilog (.NET), Logback (Java)