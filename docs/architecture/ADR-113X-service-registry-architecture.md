# ADR-113X: Arquitectura de Service Registry para Service Discovery

## Estado
✅ Aceptado

## Fecha
2025-12-10

## Contexto
Como parte de la historia US-24E (VELA-599), necesitamos implementar service discovery para microservicios en Vela. El service discovery es fundamental para arquitecturas de microservicios porque permite que los servicios se encuentren dinámicamente sin configuración hardcodeada.

## Decisión
Implementaremos una arquitectura de service registry que soporte múltiples proveedores de service discovery (Consul, Eureka, etcd, etc.) a través de una interfaz común.

### Arquitectura General
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Vela Service  │────│ ServiceRegistry  │────│  Registry Impl  │
│                 │    │   Interface      │    │  (Consul/Eureka)│
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌────────────────────┐
                    │  Service Metadata  │
                    │  - ID              │
                    │  - Name            │
                    │  - Address         │
                    │  - Port            │
                    │  - Health Status   │
                    │  - Tags            │
                    │  - Metadata        │
                    └────────────────────┘
```

### Componentes Principales

#### 1. ServiceRegistry Interface
```vela
interface ServiceRegistry {
    async fn register(service: ServiceInfo) -> Result<(), RegistryError>
    async fn deregister(serviceId: String) -> Result<(), RegistryError>
    async fn discover(serviceName: String) -> Result<List<ServiceInstance>, RegistryError>
    async fn health_check(serviceId: String) -> Result<HealthStatus, RegistryError>
    async fn watch(serviceName: String) -> Result<ServiceWatcher, RegistryError>
}
```

#### 2. ServiceInfo Struct
```vela
struct ServiceInfo {
    id: String,
    name: String,
    address: String,
    port: Number,
    tags: List<String>,
    metadata: Dict<String, String>,
    health_check: Option<HealthCheckConfig>
}
```

#### 3. Implementaciones Específicas
- **ConsulRegistry**: Integración con HashiCorp Consul
- **EurekaRegistry**: Integración con Netflix Eureka
- **EtcdRegistry**: Integración con etcd
- **InMemoryRegistry**: Para testing y desarrollo

### Health Checks
Implementaremos health checks automáticos con diferentes estrategias:
- HTTP endpoints (/health/live, /health/ready)
- TCP connections
- Custom health check functions

### Service Discovery Client
Cliente que automáticamente:
- Registra servicios al iniciar
- Actualiza health status
- Descubre servicios por nombre
- Maneja failover automáticamente

## Consecuencias

### Positivas
- ✅ **Flexibilidad**: Soporte para múltiples proveedores de service discovery
- ✅ **Escalabilidad**: Arquitectura preparada para crecimiento
- ✅ **Reliability**: Health checks y failover automático
- ✅ **Developer Experience**: API simple y consistente
- ✅ **Testing**: Fácil mocking con InMemoryRegistry

### Negativas
- ❌ **Complejidad**: Múltiples implementaciones a mantener
- ❌ **Dependencias**: Nuevas dependencias externas (HTTP clients, etc.)
- ❌ **Performance**: Overhead de network calls para discovery

## Alternativas Consideradas

### 1. **Service Discovery Embebido (Rechazado)**
- Implementar nuestro propio service discovery
- **Razón de rechazo**: No reinventar la rueda, mejor integrar con soluciones probadas

### 2. **Solo Consul (Rechazado)**
- Solo soporte para HashiCorp Consul
- **Razón de rechazo**: Limita opciones de despliegue, no todos usan Consul

### 3. **Discovery por DNS (Rechazado)**
- Usar DNS SRV records
- **Razón de rechazo**: No soporta health checks avanzados ni metadata rica

## Implementación
Ver código en: `packages/service-discovery/`

## Referencias
- Jira: [VELA-599](https://velalang.atlassian.net/browse/VELA-599)
- Historia: [US-24E](https://velalang.atlassian.net/browse/US-24E)
- RFC: Service Discovery Patterns (2023)</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-113X-service-registry-architecture.md