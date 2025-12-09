# ADR-067: Navigation API Architecture

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Después de implementar el Router widget con soporte para rutas dinámicas (TASK-066), necesitamos una API de alto nivel que permita a los desarrolladores navegar programáticamente de manera conveniente. La Navigation API debe proporcionar métodos intuitivos como `push()`, `pop()`, `replace()`, etc., abstrayendo los detalles del sistema de routing subyacente.

## Decisión
Implementaremos una Navigation API como una capa de abstracción sobre el Router widget, proporcionando:

1. **NavigationService**: Servicio singleton para navegación global
2. **Navigation API methods**: push, pop, replace, go, canGoBack, canGoForward
3. **Route builders**: Métodos helper para construir rutas con parámetros
4. **Navigation guards**: Sistema de guards para controlar navegación
5. **Integration con Router**: Comunicación bidireccional con Router widget

## Consecuencias

### Positivas
- ✅ API intuitiva y fácil de usar para desarrolladores
- ✅ Abstracción completa del sistema de routing subyacente
- ✅ Type safety en navegación con parámetros
- ✅ Navigation guards para control de acceso
- ✅ Integración perfecta con Router widget existente

### Negativas
- ❌ Capa adicional de abstracción que puede añadir complejidad
- ❌ Dependencia del Router widget para funcionar

## Alternativas Consideradas

### 1. Navigation integrada en Router widget
**Rechazada porque:** Separación de responsabilidades - el Router maneja matching y rendering, Navigation maneja la lógica de navegación.

### 2. Navigation como funciones globales
**Rechazada porque:** No permite inyección de dependencias ni testing fácil.

### 3. Navigation como parte del BuildContext
**Rechazada porque:** Limita la navegación a contextos de widget, no permite navegación global.

## Implementación

### NavigationService
```rust
pub struct NavigationService {
    router: Arc<Mutex<Router<...>>>,
    history: Vec<NavigationEntry>,
    current_index: usize,
}

impl NavigationService {
    pub fn push(&self, route: &str) -> Result<(), NavigationError>
    pub fn pop(&self) -> Result<(), NavigationError>
    pub fn replace(&self, route: &str) -> Result<(), NavigationError>
    pub fn go(&self, delta: i32) -> Result<(), NavigationError>
    pub fn can_go_back(&self) -> bool
    pub fn can_go_forward(&self) -> bool
}
```

### Route Builders
```rust
pub trait RouteBuilder {
    fn build_path(&self, params: &HashMap<String, String>) -> String;
}

impl RouteBuilder for &str {
    fn build_path(&self, params: &HashMap<String, String>) -> String {
        // Implementación de interpolación de parámetros
    }
}
```

### Navigation Guards
```rust
pub trait NavigationGuard {
    fn can_activate(&self, route: &str, context: &NavigationContext) -> bool;
}

pub struct AuthGuard;
impl NavigationGuard for AuthGuard {
    fn can_activate(&self, route: &str, context: &NavigationContext) -> bool {
        // Verificar autenticación
    }
}
```

## Referencias
- Jira: [VELA-067](https://velalang.atlassian.net/browse/VELA-067)
- Historia: [VELA-066](https://velalang.atlassian.net/browse/VELA-066)
- Documentación: TASK-066 Router Widget Implementation
- Patrón: Navigation API inspirado en React Router y Vue Router