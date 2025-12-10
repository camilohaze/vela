# Ejemplo: Foreign Language Bindings - Lodash JS Integration

Este ejemplo demuestra cómo integrar librerías JavaScript existentes en Vela usando el sistema de bindings declarativos.

## 🎯 Objetivo

Mostrar cómo Vela puede reutilizar el vasto ecosistema de JavaScript (npm) mientras mantiene su paradigma funcional puro.

## 🔨 Implementación

### 1. Definición del Binding (`lodash-binding.vela`)

```vela
@js_binding("lodash")
module Lodash {
  @pure
  fn chunk<T>(array: List<T>, size: Number) -> List<List<T>> {
    // Contract puro: divide array en chunks
    // Implementation: llama a lodash.chunk via interop JS
  }
}
```

### 2. Uso en Código Vela (`usage-example.vela`)

```vela
import 'bindings:js/lodash'

fn main() -> void {
  chunks = Lodash.chunk([1, 2, 3, 4, 5], 2)
  print("Chunks: ${chunks}") // [[1, 2], [3, 4], [5]]
}
```

## 🏗️ Arquitectura

### Cómo Funciona

1. **Declaración Pura**: Las funciones se declaran con contratos funcionales puros
2. **Implementación Impura**: El compilador genera código glue que llama al JS real
3. **Aislamiento**: Los bindings son tratados como "efectos controlados"

### Código Glue Generado (JS Backend)

```javascript
// Generado automáticamente por el compilador Vela
const lodash = require('lodash');

function vela_lodash_chunk(array, size) {
  return lodash.chunk(array, size);
}
```

## ✅ Beneficios

- **Reutilización**: Acceso a 1.5M+ paquetes npm
- **Pureza**: Contratos funcionales garantizan comportamiento predecible
- **Performance**: Zero-cost abstractions cuando es posible
- **Type Safety**: Types de Vela se mapean a types de JS

## 🚀 Ejecutar el Ejemplo

```bash
# Compilar con backend JS
vela build --target js examples/js-bindings/

# Ejecutar
node dist/usage-example.js
```

## 📚 Referencias

- **ADR**: `docs/architecture/ADR-XXX-foreign-language-bindings.md`
- **Jira**: TASK-103 (vela install), EPIC-10 (Web Backend)
- **Historia**: US-23 (Package Manager)

## 🔗 Próximos Pasos

1. Implementar generador de código glue en `compiler/bindings/`
2. Agregar soporte en runtime JS: `runtime/js/interop/`
3. Tests unitarios en `tests/unit/test_bindings.rs`
4. Documentación completa en `docs/tooling/bindings.md`