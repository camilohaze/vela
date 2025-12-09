# TASK-067: Implementar Navigation API

## 📋 Información General
- **Historia:** VELA-066 (Router widget con rutas dinámicas)
- **Estado:** En curso 🚧
- **Fecha:** 2025-01-30
- **Dependencias:** TASK-066 (Router widget)

## 🎯 Objetivo
Implementar una Navigation API de alto nivel que proporcione métodos convenientes para navegación programática, abstrayendo los detalles del Router widget subyacente.

## 🔨 Implementación Técnica

### Arquitectura
La Navigation API consistirá de:

1. **NavigationService**: Servicio central para navegación
2. **Navigation methods**: push, pop, replace, go
3. **Route builders**: Construcción de rutas con parámetros
4. **Navigation guards**: Control de acceso a rutas
5. **Integration layer**: Comunicación con Router widget

### Componentes a Implementar

#### 1. NavigationService (`runtime/ui/src/navigation/service.rs`)
```rust
pub struct NavigationService<T, F> {
    router: Arc<Mutex<Router<T, F>>>,
    history: Vec<NavigationEntry>,
    current_index: usize,
    guards: Vec<Box<dyn NavigationGuard>>,
}

impl<T, F> NavigationService<T, F> {
    pub fn new(router: Arc<Mutex<Router<T, F>>>) -> Self { ... }

    pub fn push(&self, path: &str) -> Result<(), NavigationError> { ... }
    pub fn push_with_params(&self, path: &str, params: HashMap<String, String>) -> Result<(), NavigationError> { ... }

    pub fn pop(&self) -> Result<(), NavigationError> { ... }

    pub fn replace(&self, path: &str) -> Result<(), NavigationError> { ... }
    pub fn replace_with_params(&self, path: &str, params: HashMap<String, String>) -> Result<(), NavigationError> { ... }

    pub fn go(&self, delta: i32) -> Result<(), NavigationError> { ... }
    pub fn go_back(&self) -> Result<(), NavigationError> { ... }
    pub fn go_forward(&self) -> Result<(), NavigationError> { ... }

    pub fn can_go_back(&self) -> bool { ... }
    pub fn can_go_forward(&self) -> bool { ... }

    pub fn get_current_path(&self) -> String { ... }
    pub fn get_history(&self) -> &[NavigationEntry] { ... }
}
```

#### 2. NavigationEntry (`runtime/ui/src/navigation/mod.rs`)
```rust
#[derive(Debug, Clone)]
pub struct NavigationEntry {
    pub path: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub timestamp: SystemTime,
}
```

#### 3. NavigationError (`runtime/ui/src/navigation/mod.rs`)
```rust
#[derive(Debug, Clone)]
pub enum NavigationError {
    RouteNotFound(String),
    GuardBlocked(String),
    InvalidPath(String),
    RouterNotAvailable,
}
```

#### 4. Navigation Guards
```rust
pub trait NavigationGuard {
    fn can_activate(&self, entry: &NavigationEntry, context: &NavigationContext) -> bool;
}

pub struct NavigationContext {
    pub current_entry: Option<NavigationEntry>,
    pub target_entry: NavigationEntry,
}
```

#### 5. Route Builders
```rust
pub trait RouteBuilder {
    fn build_path(&self, params: &HashMap<String, String>) -> Result<String, NavigationError>;
}

impl RouteBuilder for &str {
    fn build_path(&self, params: &HashMap<String, String>) -> Result<String, NavigationError> {
        // Implementar interpolación de parámetros
        // Ej: "/users/:id" + {id: "123"} = "/users/123"
    }
}
```

### API Pública

#### Uso Básico
```rust
// Inicialización
let navigation = NavigationService::new(router);

// Navegación básica
navigation.push("/home")?;
navigation.push("/users/123")?;
navigation.pop()?;
navigation.go_back()?;

// Navegación con parámetros
let params = HashMap::from([("id".to_string(), "123".to_string())]);
navigation.push_with_params("/users/:id", params)?;

// Navegación con query
navigation.push("/search?q=rust&page=1")?;
```

#### Guards
```rust
struct AuthGuard;
impl NavigationGuard for AuthGuard {
    fn can_activate(&self, entry: &NavigationEntry, context: &NavigationContext) -> bool {
        // Verificar si usuario está autenticado
        is_authenticated()
    }
}

navigation.add_guard(Box::new(AuthGuard));
```

### Tests Requeridos

#### Unit Tests
- ✅ `test_navigation_push_pop`: Push y pop básico
- ✅ `test_navigation_replace`: Replace functionality
- ✅ `test_navigation_go`: Go forward/backward
- ✅ `test_navigation_with_params`: Navegación con parámetros
- ✅ `test_navigation_guards`: Guards bloqueando navegación
- ✅ `test_route_building`: Route builders
- ✅ `test_navigation_errors`: Manejo de errores

#### Integration Tests
- ✅ `test_navigation_with_router`: Integración con Router widget
- ✅ `test_navigation_history`: Manejo correcto del history

### Consideraciones Técnicas

#### Thread Safety
- NavigationService debe ser thread-safe para uso concurrente
- Usar Arc<Mutex<>> para compartir estado entre threads

#### Error Handling
- NavigationError enum para diferentes tipos de errores
- Result<> para operaciones que pueden fallar

#### Performance
- History limitado a tamaño razonable (ej: 50 entradas)
- Lazy evaluation de guards
- Efficient parameter interpolation

### Dependencias
- **TASK-066**: Router widget (ya implementado)
- **std::collections::HashMap**: Para parámetros y query
- **std::sync**: Para thread safety

### Métricas de Calidad
- **Coverage**: >= 90% test coverage
- **Performance**: < 1ms para operaciones básicas
- **Memory**: < 10KB por NavigationService instance
- **API Completeness**: 100% de métodos especificados

## ✅ Criterios de Aceptación
- [x] NavigationService implementado con todos los métodos
- [x] Integración completa con Router widget
- [x] Navigation guards funcionando
- [x] Route builders implementados
- [x] Tests exhaustivos (8+ tests)
- [x] Documentación completa
- [x] Error handling robusto
- [x] Thread safety garantizada

## 🔗 Referencias
- **Jira:** [VELA-067](https://velalang.atlassian.net/browse/VELA-067)
- **Historia:** [VELA-066](https://velalang.atlassian.net/browse/VELA-066)
- **ADR:** [ADR-067](../architecture/ADR-067-navigation-api.md)
- **Router:** Ver implementación en `runtime/ui/src/navigation/router.rs`