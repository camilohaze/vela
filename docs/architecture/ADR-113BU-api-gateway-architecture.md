# ADR-113BU: Arquitectura del API Gateway

## Estado
✅ Aceptado

## Fecha
2024-01-15

## Contexto
Necesitamos implementar un API Gateway para Vela que proporcione:
- Routing dinámico a diferentes microservicios
- Load balancing con múltiples estrategias
- Rate limiting por IP, usuario y endpoint
- Autenticación y autorización centralizada
- Observabilidad y métricas
- Configuración declarativa con decoradores

El gateway debe ser:
- Alto rendimiento (basado en Rust/Tokio)
- Configurable en tiempo de compilación
- Integrable con el sistema de configuración existente
- Extensible con plugins/middleware

## Decisión
Implementar API Gateway como un servicio independiente con arquitectura modular:

### Arquitectura General
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client        │────│   API Gateway    │────│   Services      │
│                 │    │                  │    │                 │
│ - HTTP/HTTPS    │    │ - Routing        │    │ - Microservices │
│ - WebSocket     │    │ - Load Balancing │    │ - Databases     │
│ - gRPC          │    │ - Rate Limiting  │    │ - External APIs │
└─────────────────┘    │ - Auth/Authz     │    └─────────────────┘
                       │ - Observability  │
                       │ - Plugins        │
                       └──────────────────┘
```

### Componentes Principales

#### 1. Router (Routing Engine)
- **Tecnología**: Trie-based routing con wildcards y parámetros
- **Funcionalidad**: Matching de paths, extracción de parámetros, middleware chaining
- **Configuración**: Declarativa con decoradores @route

#### 2. Load Balancer
- **Estrategias**:
  - Round-robin
  - Least-connections
  - Weighted random
  - IP hash
- **Health checks**: HTTP/TCP con circuit breaker
- **Failover**: Automático con retry logic

#### 3. Rate Limiter
- **Algoritmos**:
  - Token bucket
  - Leaky bucket
  - Fixed window
  - Sliding window
- **Scopes**: Global, por ruta, por IP, por usuario
- **Backend**: Redis para estado distribuido

#### 4. Auth/Authz Engine
- **Protocolos**: JWT, OAuth2, API Keys, mTLS
- **Integración**: Con servicios de identidad existentes
- **Cache**: LRU cache para tokens validados

#### 5. Observability
- **Métricas**: Prometheus-compatible
- **Logs**: Structured logging con correlation IDs
- **Tracing**: OpenTelemetry integration
- **Health checks**: Endpoints para monitoring

### Configuración Declarativa
```vela
@gateway({
  port: 8080,
  tls: true,
  rateLimit: "1000req/min",
  auth: "jwt"
})
class ApiGateway {
  // Routes se definen con decoradores
}

@route("/api/v1/users", methods: ["GET", "POST"])
@rateLimit("100req/min")
@auth("required")
async fn handleUsers(req: Request) -> Response {
  // Routing logic
}
```

### Plugins/Middleware System
- **Arquitectura**: Chain of responsibility pattern
- **Tipos de plugins**:
  - Pre-routing (auth, rate limiting)
  - Post-routing (logging, metrics)
  - Error handling
  - Custom business logic

## Consecuencias

### Positivas
- **Escalabilidad**: Arquitectura modular permite escalar componentes individualmente
- **Flexibilidad**: Sistema de plugins permite extensiones sin modificar core
- **Observabilidad**: Métricas integradas facilitan debugging y monitoring
- **Type Safety**: Configuración declarativa con validación compile-time
- **Performance**: Implementación en Rust garantiza alto rendimiento

### Negativas
- **Complejidad**: Arquitectura distribuida aumenta complejidad operacional
- **Dependencias**: Requiere Redis para rate limiting distribuido
- **Latencia**: Middleware chain agrega overhead por request
- **Configuración**: Curva de aprendizaje para configuración avanzada

## Alternativas Consideradas

### 1. Kong Gateway (Rechazado)
- **Pros**: Madura, feature-complete, gran comunidad
- **Cons**: No integrado con Vela, configuración externa, no type-safe
- **Razón**: Necesitamos integración nativa con el ecosistema Vela

### 2. Traefik (Rechazado)
- **Pros**: Auto-discovery, Kubernetes-native
- **Cons**: Go-based, no integración con Vela compiler
- **Razón**: Necesitamos decoradores y configuración compile-time

### 3. Nginx + Lua (Rechazado)
- **Pros**: Alto rendimiento, extensible con Lua
- **Cons**: Configuración imperativa, no type-safe, complejo de mantener
- **Razón**: Paradigma no se alinea con filosofía funcional de Vela

## Implementación
Ver código en: `compiler/src/gateway_*.rs`

## Referencias
- Jira: VELA-611
- ADR-113BP: Config Management Architecture
- Documentación: docs/features/VELA-611/