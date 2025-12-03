# ADR-306: Elección del Framework HTTP para Vela Runtime

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto
Necesitamos migrar el framework HTTP de Python a Rust como parte de EPIC-RUST-04. El framework HTTP debe proporcionar:

- Servidor HTTP asíncrono de alto rendimiento
- Cliente HTTP robusto
- Sistema de routing flexible
- Middleware system extensible
- Soporte para TLS/SSL
- Integración con el sistema de eventos existente

## Decisión
Usaremos **hyper** como base del framework HTTP, complementado con **reqwest** para el cliente HTTP.

### Arquitectura Elegida
```
vela_http/
├── server/          # HTTP server usando hyper
├── client/          # HTTP client usando reqwest
├── routing/         # Sistema de routing personalizado
├── middleware/      # Sistema de middleware
└── types/          # Request/Response types
```

## Consecuencias

### Positivas
- **Performance:** hyper es uno de los servidores HTTP más rápidos en Rust
- **Flexibilidad:** hyper es de bajo nivel, permitiendo control total
- **Ecosistema:** reqwest es el cliente HTTP más popular en Rust
- **Mantenimiento:** Ambas crates son mantenidas activamente
- **Integración:** Fácil integración con tokio (nuestro runtime async)

### Negativas
- **Complejidad:** hyper es de bajo nivel, requiere más código boilerplate
- **Aprendizaje:** Curva de aprendizaje más pronunciada vs frameworks de alto nivel
- **Tiempo:** Mayor tiempo de desarrollo inicial

## Alternativas Consideradas

### 1. Actix-Web
**Descripción:** Framework web de alto nivel para Rust
**Pros:**
- Alto nivel de abstracción
- Rápido desarrollo
- Buena documentación
- Actor system integrado

**Cons:**
- Dependencia de actor system (conflicto con nuestro sistema de eventos)
- Menos control sobre detalles de bajo nivel
- Mayor consumo de memoria

**Rechazada porque:** Conflicto con nuestro sistema de eventos personalizado y mayor overhead.

### 2. Rocket
**Descripción:** Framework web tipo Ruby on Rails para Rust
**Pros:**
- Sintaxis elegante
- Type-safe routing
- Buena ergonomía

**Cons:**
- No es async-first (usa threads)
- Menor performance en escenarios de alta concurrencia
- Comunidad más pequeña

**Rechazada porque:** No es async-first, incompatible con nuestro runtime tokio.

### 3. Warp
**Descripción:** Framework funcional para HTTP en Rust
**Pros:**
- Programación funcional
- Composición fácil
- Buen performance

**Cons:**
- Comunidad pequeña
- Menos maduro
- Menos crates auxiliares

**Rechazada porque:** Comunidad pequeña y menos madurez comparado con hyper.

### 4. Axum
**Descripción:** Framework web ergonómico basado en tower/hyper
**Pros:**
- Alto nivel sobre hyper
- Excelente ergonomía
- Basado en tower ecosystem

**Cons:**
- Más reciente, menos battle-tested
- Comunidad en crecimiento

**Rechazada porque:** Preferimos control directo sobre hyper para optimizaciones específicas.

## Implementación
Ver código en: `src/vela_http/`

## Referencias
- Jira: [TASK-RUST-306](https://velalang.atlassian.net/browse/TASK-RUST-306)
- Epic: [EPIC-RUST-04](https://velalang.atlassian.net/browse/EPIC-RUST-04)
- Documentación hyper: https://hyper.rs/
- Documentación reqwest: https://docs.rs/reqwest/

## Implementación Técnica

### Dependencias
```toml
[dependencies]
hyper = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
regex = "1.10"
```

### Arquitectura de Alto Nivel
```rust
// Server
let server = HttpServer::new()
    .route(Method::GET, "/api/users", get_users_handler)
    .middleware(LoggingMiddleware)
    .middleware(AuthMiddleware)
    .serve("127.0.0.1:8080")
    .await?;

// Client
let client = HttpClient::new();
let response = client.get("http://api.example.com/users").await?;
```