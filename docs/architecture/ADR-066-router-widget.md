# ADR-066: Arquitectura del Router Widget

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Necesitamos implementar un sistema de navegación y routing para aplicaciones Vela. El Router widget debe permitir:

- Definición declarativa de rutas
- Soporte para rutas dinámicas con parámetros
- Navegación programática (push, pop, replace)
- Manejo de rutas anidadas
- Integración con el sistema reactivo de Vela

## Decisión
Implementaremos un Router widget basado en una arquitectura de rutas declarativas con las siguientes características:

### Arquitectura Elegida
- **Router Widget**: Widget contenedor que maneja el estado de navegación
- **Route Definition**: Sistema declarativo de rutas con pattern matching
- **Navigation Context**: Context API para acceso a navegación desde cualquier widget
- **History Management**: Stack-based navigation con push/pop/replace operations

### Componentes Principales
1. **Router<T>**: Widget principal que maneja el estado de rutas
2. **Route<T>**: Define una ruta individual con pattern y builder
3. **NavigationContext**: Proporciona acceso a funciones de navegación
4. **RouteMatch**: Resultado del matching de una ruta con parámetros extraídos

## Consecuencias

### Positivas
- ✅ Routing declarativo y type-safe
- ✅ Soporte completo para rutas dinámicas
- ✅ Integración perfecta con sistema reactivo
- ✅ Navegación programática flexible
- ✅ Manejo de rutas anidadas

### Negativas
- ❌ Complejidad adicional en el sistema de widgets
- ❌ Curva de aprendizaje para definición de rutas
- ❌ Overhead de pattern matching en cada navegación

## Alternativas Consideradas

### 1. Router Basado en Strings (Rechazado)
```rust
// ❌ MENOS TYPE-SAFE
Router::new(vec![
    Route::new("/users/:id", user_page_builder),
    Route::new("/posts/:slug", post_page_builder),
])
```
**Razones de rechazo:**
- Sin type safety en parámetros de ruta
- Errores de runtime en lugar de compile-time
- Dificultad para refactorizar rutas

### 2. Router con Enums (Aceptado)
```rust
// ✅ TYPE-SAFE
enum Route {
    Home,
    User { id: UserId },
    Post { slug: String },
}

Router::new(|route: Route| match route {
    Route::Home => HomePage::new(),
    Route::User { id } => UserPage::new(id),
    Route::Post { slug } => PostPage::new(slug),
})
```
**Razones de aceptación:**
- Type safety completo
- Errores en compile-time
- Refactoring seguro
- Mejor ergonomía

### 3. Router Global Singleton (Rechazado)
**Razones de rechazo:**
- Estado global mutable
- Dificultad para testing
- No composable
- Problemas de concurrencia

## Implementación
Ver código en: `runtime/ui/src/navigation/router.rs`

## Referencias
- Jira: [VELA-066](https://velalang.atlassian.net/browse/VELA-066)
- Documentación: [Router Widget API](../../features/VELA-066/TASK-066.md)