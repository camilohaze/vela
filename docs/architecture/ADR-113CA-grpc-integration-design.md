# ADR-113CA: Arquitectura de Integración gRPC en Vela

## Estado
✅ Aceptado

## Fecha
2025-12-30

## Contexto
Como desarrollador, necesito implementar soporte gRPC en Vela para comunicación eficiente entre servicios. gRPC ofrece ventajas significativas sobre REST:

- **Protocol Buffers**: Serialización binaria eficiente vs JSON
- **Streaming**: Soporte nativo para server/client/bidirectional streaming
- **Type Safety**: Generación de código con tipos estrictos
- **Performance**: Mejor throughput y menor latencia
- **Multi-language**: Soporte para múltiples lenguajes

El desafío es integrar gRPC de manera idiomática con el paradigma funcional puro de Vela, manteniendo la simplicidad y la seguridad de tipos.

## Decisión
Implementar gRPC support en Vela con la siguiente arquitectura:

### 1. **Decoradores Arquitectónicos**
```vela
@grpc.service(name="UserService", package="vela.user.v1")
service UserService {
  @grpc.method
  async fn getUser(@grpc.field request: GetUserRequest) -> Result<UserResponse>
  
  @grpc.method(streaming=server)
  async fn listUsers(@grpc.field request: ListUsersRequest) -> Stream<UserResponse>
}
```

### 2. **Sistema de Tipos gRPC**
- `GrpcService<T>`: Interface base para servicios
- `GrpcMethod<T, R>`: Tipo para métodos gRPC
- `Stream<T>`: Tipo para streaming responses
- `GrpcClient<T>`: Cliente generado automáticamente

### 3. **Integración con Runtime**
- **Server**: Integración con el API Gateway existente
- **Client**: Cliente HTTP/2 con connection pooling
- **Streaming**: Soporte para todos los tipos de streaming
- **Middleware**: Interceptors para logging, auth, metrics

### 4. **Code Generation**
- Generación automática desde archivos `.proto`
- Decoradores para metadata adicional
- Validación de tipos en compile-time

## Consecuencias

### Positivas
- ✅ **Performance**: 5-10x mejor que REST/JSON
- ✅ **Type Safety**: Validación estricta de contratos
- ✅ **Streaming**: Soporte completo para comunicación en tiempo real
- ✅ **Ecosystem**: Compatibilidad con servicios existentes
- ✅ **Developer Experience**: Generación automática de código

### Negativas
- ❌ **Complejidad**: Mayor complejidad vs HTTP simple
- ❌ **Dependencias**: Requiere protobuf compiler
- ❌ **Learning Curve**: Nuevos conceptos (proto files, streaming)
- ❌ **Debugging**: Más difícil debuggear que HTTP

## Alternativas Consideradas

### 1. **REST + JSON (Rechazada)**
- **Pros**: Simple, familiar, herramientas existentes
- **Cons**: Menor performance, no streaming nativo
- **Razón de rechazo**: No cumple con requerimiento de "comunicación eficiente"

### 2. **GraphQL (Rechazada)**
- **Pros**: Flexible queries, type safety
- **Cons**: Overhead de query parsing, complejidad adicional
- **Razón de rechazo**: No optimizado para servicio-a-servicio

### 3. **WebSockets + JSON (Rechazada)**
- **Pros**: Bidirectional, real-time
- **Cons**: No type safety, protocolo custom
- **Razón de rechazo**: Falta estructura y eficiencia

### 4. **Custom Binary Protocol (Rechazada)**
- **Pros**: Optimizado para Vela
- **Cons**: No interoperable, reinventar la rueda
- **Razón de rechazo**: Aislamiento del ecosystem

## Implementación

### Fase 1: Core Infrastructure
1. **Protobuf Parser**: Parser para archivos `.proto`
2. **Code Generator**: Generar tipos Vela desde proto
3. **Runtime Support**: HTTP/2 client/server en runtime

### Fase 2: Decorators & Services
1. **@grpc.service**: Decorator para servicios
2. **@grpc.method**: Decorator para métodos
3. **Service Registry**: Registro automático de servicios

### Fase 3: Streaming & Advanced Features
1. **Server Streaming**: `Stream<T>` responses
2. **Client Streaming**: `Stream<T>` requests
3. **Bidirectional**: Streams en ambas direcciones
4. **Interceptors**: Middleware para gRPC calls

### Fase 4: Tooling & Integration
1. **CLI Commands**: `vela grpc generate`, `vela grpc serve`
2. **API Gateway**: Routing automático para servicios gRPC
3. **Monitoring**: Métricas y tracing integrado

## Referencias
- **Jira**: [VELA-1080](https://velalang.atlassian.net/browse/VELA-1080)
- **Epic**: EPIC-09N: gRPC Support
- **Documentación**: gRPC official documentation
- **Inspiración**: NestJS gRPC, Spring Boot gRPC

## Implementación Técnica

### Arquitectura de Capas
```
┌─────────────────┐
│   Vela Code     │ ← @grpc.service, @grpc.method
├─────────────────┤
│ Code Generator  │ ← Genera tipos desde .proto
├─────────────────┤
│  gRPC Runtime   │ ← HTTP/2, protobuf, streaming
├─────────────────┤
│   Transport     │ ← TCP, TLS, connection pooling
└─────────────────┘
```

### Integration Points
- **Compiler**: Parsing de decoradores gRPC
- **Runtime**: Ejecución de servicios gRPC
- **API Gateway**: Routing y load balancing
- **Tooling**: Code generation y CLI

### Security Considerations
- TLS obligatorio para producción
- Authentication via interceptors
- Authorization checks en cada método
- Rate limiting integrado con gateway