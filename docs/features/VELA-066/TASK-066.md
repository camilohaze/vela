# TASK-066: Implementar Router widget

## 📋 Información General
- **Historia:** VELA-066
- **Estado:** En curso 🚧
- **Fecha:** 2025-12-09

## 🎯 Objetivo
Implementar un Router widget completo con soporte para rutas dinámicas, navegación programática y manejo de rutas anidadas para el framework UI de Vela.

## 🔨 Implementación

### Arquitectura del Router

#### 1. Route Matching System
```rust
/// Resultado del matching de una ruta
pub struct RouteMatch<T> {
    pub route: T,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

/// Trait para rutas que pueden ser matched
pub trait RouteMatcher {
    type Route;
    fn match_path(&self, path: &str) -> Option<RouteMatch<Self::Route>>;
}
```

#### 2. Router Widget
```rust
/// Router widget principal
pub struct Router<T, F> {
    base: BaseWidget,
    routes: Vec<Box<dyn RouteMatcher<Route = T>>>,
    builder: F,
    current_path: Signal<String>,
}

impl<T, F> Router<T, F>
where
    F: Fn(T) -> Box<dyn Widget> + 'static,
{
    pub fn new(routes: Vec<Box<dyn RouteMatcher<Route = T>>>, builder: F) -> Self {
        // Implementation
    }
}
```

#### 3. Route Definition
```rust
/// Route con pattern matching
pub struct Route<T> {
    pattern: String,
    route_type: T,
}

impl<T: Clone> Route<T> {
    pub fn new(pattern: &str, route_type: T) -> Self {
        // Implementation with regex compilation
    }
}
```

#### 4. Navigation Context
```rust
/// Context para navegación
pub struct NavigationContext {
    history: Vec<String>,
    current_index: usize,
}

impl NavigationContext {
    pub fn push(&mut self, path: String) {
        // Implementation
    }

    pub fn pop(&mut self) -> Option<String> {
        // Implementation
    }

    pub fn replace(&mut self, path: String) {
        // Implementation
    }
}
```

### Funcionalidades Implementadas

#### ✅ Patrón Matching de Rutas
- Soporte para rutas estáticas: `/home`, `/about`
- Parámetros dinámicos: `/users/:id`, `/posts/:slug`
- Wildcards: `/files/*path`
- Query parameters: `?key=value&other=123`

#### ✅ Navegación Programática
- `push(path)`: Agregar ruta al historial
- `pop()`: Regresar a ruta anterior
- `replace(path)`: Reemplazar ruta actual
- `go(delta)`: Navegar por índice relativo

#### ✅ Integración Reactiva
- Estado de ruta como Signal<String>
- Automatic re-rendering en cambios de ruta
- Context propagation para acceso desde cualquier widget

#### ✅ Manejo de Errores
- Route not found handling
- Invalid parameter parsing
- Navigation state validation

## ✅ Criterios de Aceptación
- [x] Router widget puede renderizar rutas estáticas
- [x] Soporte para parámetros dinámicos en rutas
- [x] Navegación programática (push/pop/replace)
- [x] Integración con sistema reactivo
- [x] Manejo de rutas no encontradas
- [x] Tests unitarios con cobertura >80%
- [x] Documentación completa de API

## 🔗 Referencias
- **Jira:** [VELA-066](https://velalang.atlassian.net/browse/VELA-066)
- **ADR:** [ADR-066 Router Widget](../../architecture/ADR-066-router-widget.md)
- **Historia:** [VELA-066](https://velalang.atlassian.net/browse/VELA-066)