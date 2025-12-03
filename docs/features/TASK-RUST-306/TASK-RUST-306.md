# TASK-RUST-306: Migrar HTTP Framework

## 📋 Información General
- **Historia:** US-RUST-04
- **Epic:** EPIC-RUST-04
- **Estado:** En curso 🔄
- **Fecha:** 2025-12-03

## 🎯 Objetivo
Migrar completamente el framework HTTP de Python a Rust, proporcionando un servidor HTTP asíncrono de alto rendimiento y un cliente HTTP robusto.

## 🔨 Alcance Técnico

### Arquitectura del HTTP Framework

#### 1. HTTP Server (`vela_http::server`)
```rust
pub struct HttpServer {
    addr: SocketAddr,
    routes: RouteTable,
    middleware: Vec<Box<dyn Middleware>>,
    tls_config: Option<TlsConfig>,
}

impl HttpServer {
    pub async fn serve(self) -> Result<(), HttpError>;
    pub fn route(mut self, method: Method, path: &str, handler: RouteHandler) -> Self;
    pub fn middleware<M: Middleware>(mut self, middleware: M) -> Self;
}
```

#### 2. HTTP Client (`vela_http::client`)
```rust
pub struct HttpClient {
    client: reqwest::Client,
    timeout: Duration,
    user_agent: String,
}

impl HttpClient {
    pub async fn get(&self, url: &str) -> Result<Response, HttpError>;
    pub async fn post(&self, url: &str, body: Body) -> Result<Response, HttpError>;
    pub async fn request(&self, req: Request) -> Result<Response, HttpError>;
}
```

#### 3. Request/Response Types
```rust
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Body,
}

pub struct Response {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Body,
}
```

### Middleware System
```rust
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, req: Request, next: Next) -> Result<Response, HttpError>;
}

pub struct Next<'a> {
    handler: &'a dyn Handler,
}

impl<'a> Next<'a> {
    pub async fn run(mut self, req: Request) -> Result<Response, HttpError>;
}
```

### Routing System
```rust
pub struct RouteTable {
    routes: HashMap<(Method, String), Box<dyn Handler>>,
    dynamic_routes: Vec<(Method, Regex, Box<dyn Handler>)>,
}

impl RouteTable {
    pub fn insert<H: Handler>(&mut self, method: Method, path: &str, handler: H);
    pub fn find(&self, method: &Method, path: &str) -> Option<&dyn Handler>;
}
```

## 🧪 Tests Requeridos

### Unit Tests
- [ ] Request/Response parsing
- [ ] Header manipulation
- [ ] Body reading/writing
- [ ] Route matching
- [ ] Middleware chain execution

### Integration Tests
- [ ] HTTP server startup/shutdown
- [ ] Client-server communication
- [ ] Concurrent requests handling
- [ ] Error scenarios (404, 500, timeouts)

### Benchmarks
- [ ] Request throughput (req/sec)
- [ ] Latency percentiles
- [ ] Memory usage
- [ ] Connection pooling efficiency

## 📊 Métricas de Éxito
- **Performance:** > 10,000 req/sec en un solo core
- **Latency:** < 1ms p50, < 5ms p99
- **Memory:** < 50MB para 1000 conexiones concurrentes
- **Coverage:** > 80% test coverage

## 🔗 Dependencias
- `tokio` - Async runtime
- `hyper` - HTTP implementation
- `reqwest` - HTTP client
- `serde` - JSON serialization
- `regex` - Route pattern matching

## ✅ Checklist de Implementación
- [ ] ADR: Decisión de usar hyper vs actix-web
- [ ] HTTP server básico
- [ ] HTTP client básico
- [ ] Routing system
- [ ] Middleware system
- [ ] Error handling
- [ ] TLS/SSL support
- [ ] Tests completos
- [ ] Benchmarks
- [ ] Documentación
- [ ] Integration con event system

## 🚀 Próximos Pasos
1. Crear ADR para arquitectura HTTP
2. Implementar HTTP server básico
3. Implementar HTTP client básico
4. Agregar routing system
5. Integrar middleware
6. Tests y benchmarks
7. Documentación final