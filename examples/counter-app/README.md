# Ejemplo: Contador Reactivo en Vela

Este ejemplo demuestra las funcionalidades implementadas en **VELA-25: JavaScript Compilation**, mostrando cómo crear una aplicación web completa con Vela.

## 🎯 Características Demostradas

### ✅ Sistema de Componentes Reactivos
- **Componentes con estado**: `state count: Number = 0`
- **Bindings automáticos**: `{{count}}` en templates
- **Event handlers**: `@click="increment"`
- **Ciclo de vida**: `mount()`, `update()`

### ✅ Inyección de Dependencias
- **Servicios**: `@injectable service CounterService`
- **Repositorios**: `@injectable repository CounterRepository`
- **Inyección automática**: `inject(CounterRepository)`

### ✅ Arquitectura Modular
- **Módulos**: `@module({...}) module AppModule`
- **Declaraciones**: Componentes, servicios, repositorios
- **Bootstrap automático**: Framework maneja inicialización

### ✅ Generación de JavaScript
- **Compilación automática**: Vela → JavaScript
- **Runtime reactivo**: Señales y bindings automáticos
- **DOM rendering**: Componentes → elementos HTML
- **API moderna**: `localStorage`, `print()`, etc.

## 🚀 Cómo Ejecutar

```bash
# Compilar a JavaScript
vela compile examples/counter-app/main.vela --output dist/

# Ejecutar en navegador
# El archivo generado puede abrirse directamente en un navegador
# o integrarse en cualquier aplicación web
```

## 📁 Estructura del Código

```
examples/counter-app/
├── main.vela          # Código fuente Vela
└── README.md         # Esta documentación
```

## 🔧 Tecnologías Usadas

- **Vela Compiler**: Compilación a JavaScript
- **Reactive Runtime**: Sistema de señales reactivas
- **DOM Renderer**: Conversión VNode → DOM
- **Dependency Injection**: Contenedor IoC automático
- **Modern JavaScript**: ES6+ features generadas

## 🎨 Funcionalidades de UI

- **Contador interactivo** con botones + y -
- **Reset functionality** para volver a cero
- **Persistencia automática** usando localStorage
- **Updates en tiempo real** del DOM
- **Logging** de eventos del ciclo de vida

## 📊 Beneficios de Vela

1. **Sintaxis moderna**: Similar a TypeScript/Java pero más expresiva
2. **Reactividad automática**: No necesitas manualmente actualizar DOM
3. **Type safety**: Sistema de tipos fuerte
4. **Dependency injection**: Arquitectura modular y testable
5. **Compilación optimizada**: JavaScript moderno y eficiente

---

**Este ejemplo muestra el poder de Vela para crear aplicaciones web modernas con menos código y más seguridad.** ✨