# ADR-901: Extracción de Paquetes Independientes

## Estado
✅ Aceptado

## Fecha
2024-12-30

## Contexto
El proyecto Vela ha crecido significativamente y el runtime contiene múltiples sistemas modulares que pueden beneficiarse de ser paquetes independientes. Esto permitiría:

- Reutilización externa de componentes individuales
- Desarrollo paralelo de diferentes subsistemas
- Reducción del acoplamiento entre componentes
- Publicación independiente en crates.io
- Mejor organización del código

## Decisión
Extraer los siguientes sistemas del runtime como paquetes independientes de Rust:

### ✅ Completado - Core Packages (Prioridad Alta)
1. **concurrency** - Sistema de concurrencia con actores y channels
2. **http** - Framework HTTP con cliente/servidor
3. **reactive** - Sistema reactivo con signals y computed values

### ✅ Completado - Medium Priority Packages
4. **events** - Sistema de eventos pub/sub con bus de eventos
5. **di** - Contenedor de dependency injection con múltiples scopes

### 🔄 Pendiente - Low Priority Packages
6. **vela-validation** - Sistema de validación de datos
7. **vela-i18n** - Sistema de internacionalización
8. **vela-logging** - Sistema de logging estructurado

## Consecuencias

### Positivas
- ✅ **Reutilización externa**: Los paquetes pueden ser usados por otros proyectos Rust
- ✅ **Mantenimiento independiente**: Cada paquete puede tener su propio ciclo de releases
- ✅ **Desarrollo paralelo**: Diferentes equipos pueden trabajar en diferentes paquetes
- ✅ **Reducción de dependencias**: Los usuarios solo importan lo que necesitan
- ✅ **Mejor testing**: Tests más enfocados y rápidos por paquete
- ✅ **API más clara**: Interfaces bien definidas entre paquetes

### Negativas
- ⚠️ **Complejidad de coordinación**: Cambios en un paquete pueden afectar otros
- ⚠️ **Version management**: Necesidad de mantener compatibilidad entre versiones
- ⚠️ **Documentación duplicada**: Algunos conceptos se documentan en múltiples lugares

## Alternativas Consideradas

### 1. Mantener todo en el runtime (Rechazada)
- **Razón**: Limita la reutilización y hace el runtime muy grande
- **Consecuencia**: Los usuarios tendrían que incluir todo el runtime aunque solo necesiten una parte

### 2. Extraer como módulos separados pero no paquetes (Rechazada)
- **Razón**: No permite reutilización externa ni publicación independiente
- **Consecuencia**: Los beneficios se limitan solo al desarrollo interno

### 3. Crear un monorepo con workspaces (Aceptada)
- **Razón**: Permite desarrollo coordinado mientras mantiene independencia
- **Consecuencia**: Mejor balance entre modularidad y facilidad de desarrollo

## Implementación

### Estructura de Paquetes
```
packages/
├── concurrency/     # Actores, channels, async utils
├── http/           # HTTP client/server
├── reactive/       # Signals, computed, effects
├── events/         # Event bus, pub/sub
├── di/            # Dependency injection
├── validation/    # Data validation
├── i18n/          # Internationalization
└── logging/       # Structured logging
```

### Proceso de Extracción
1. **Crear directorio del paquete** en `packages/`
2. **Copiar módulos** desde `runtime/src/` a `packages/{name}/src/`
3. **Actualizar imports** de `super::` a `crate::`
4. **Crear Cargo.toml** con dependencias apropiadas
5. **Actualizar workspace Cargo.toml** para incluir el nuevo paquete
6. **Actualizar runtime Cargo.toml** para depender del paquete
7. **Actualizar runtime/src/lib.rs** para re-exportar el paquete
8. **Ejecutar tests** para verificar funcionalidad
9. **Actualizar documentación**

### Convenciones de Naming
- **Prefijo**: Todos los paquetes usan prefijo `vela-`
- **Separadores**: Usar guiones para nombres compuestos (`vela-dependency-injection` → `vela-di`)
- **Consistencia**: Mantener nombres similares a los módulos originales

## Referencias
- **Runtime**: `runtime/src/lib.rs`
- **Workspace**: `Cargo.toml`
- **Paquetes**: `packages/` directory

## Estado de Implementación

### ✅ Core Packages
- [x] concurrency: Completado
- [x] http: Completado
- [x] reactive: Completado

### ✅ Medium Priority Packages
- [x] events: Completado (4 módulos, 0 tests)
- [x] di: Completado (6 módulos, 5 tests)

### 🔄 Próximos Pasos
- [ ] Extraer remaining low-priority packages
- [ ] Crear ejemplos de uso independiente
- [ ] Publicar paquetes en crates.io
- [ ] Actualizar documentación de arquitectura