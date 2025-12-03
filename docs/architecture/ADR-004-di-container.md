# ADR-004: Arquitectura del DI Container

## Estado
✅ Aceptado

## Fecha
2025-12-03

## Contexto
Necesitamos migrar el sistema de Dependency Injection (DI) de Python a Rust para EPIC-RUST-04. El sistema actual en Python tiene limitaciones en performance, type safety y memory management. Rust nos permite implementar un DI container con garantías de memoria y concurrencia.

## Decisión
Implementaremos un DI container en Rust con la siguiente arquitectura:

### 1. Provider Pattern
Usaremos un trait `Provider<T>` que define cómo crear instancias de servicios, permitiendo diferentes estrategias de creación (singleton, transient, factory).

### 2. Scope-based Lifetime Management
Implementaremos scopes (Singleton, Scoped, Transient) para controlar el lifecycle de las dependencias.

### 3. Constructor Injection Automática
La resolución de dependencias será automática basada en los tipos de parámetros del constructor.

### 4. Thread Safety
Todo el container será `Send + Sync` para uso seguro en entornos multi-threaded.

## Consecuencias

### Positivas
- **Type Safety**: Garantías de tipos en tiempo de compilación
- **Performance**: Resolución de dependencias < 100μs
- **Memory Safety**: Zero memory leaks, lifetimes manejados correctamente
- **Thread Safety**: Seguro para uso concurrente
- **Flexibility**: Múltiples estrategias de providers

### Negativas
- **Complexity**: Mayor complejidad que implementación Python
- **Learning Curve**: Nuevos conceptos (traits, generics, lifetimes)
- **Code Generation**: Necesidad de derive macros para inyección automática

## Alternativas Consideradas

### 1. Runtime Reflection (Rechazada)
**Decisión**: No usar reflexión en runtime
**Razones**:
- Rust no tiene reflexión como Java/C#
- Alternativas como `inventory` crate agregan complejidad
- Pierde type safety en tiempo de compilación

### 2. Manual Registration Only (Rechazada)
**Decisión**: Mantener registro automático cuando sea posible
**Razones**:
- La ergonomía es importante para developers
- Constructor injection es más declarative
- Reduce boilerplate code

### 3. No Scopes (Rechazada)
**Decisión**: Implementar scopes completos
**Razones**:
- Scopes son esenciales para control de lifecycle
- Singleton/Transient son patterns comunes
- Scoped es útil para web requests

## Implementación

### Provider Trait
```rust
pub trait Provider<T: 'static>: Send + Sync {
    fn provide(&self, container: &DIContainer) -> Result<T, DIError>;
    fn scope(&self) -> Scope;
}
```

### DI Container
```rust
pub struct DIContainer {
    services: HashMap<TypeId, ServiceDescriptor>,
    singletons: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    scoped_instances: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}
```

### Service Registration
```rust
impl DIContainer {
    pub fn register_singleton<T: 'static + Send + Sync>(&mut self) -> Result<(), DIError>
    pub fn register_transient<T: 'static>(&mut self) -> Result<(), DIError>
    pub fn register_factory<F, T>(&mut self, factory: F) -> Result<(), DIError>
        where F: Fn(&DIContainer) -> Result<T, DIError> + Send + Sync + 'static
}
```

## Referencias
- Jira: TASK-RUST-304
- Documentación: docs/features/TASK-RUST-304/
- Código: runtime/src/di/