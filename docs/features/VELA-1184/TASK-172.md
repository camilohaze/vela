# TASK-172: Implementar Dead Code Elimination

## 📋 Información General
- **Historia:** VELA-1184 (Performance Optimizations)
- **Estado:** En desarrollo 🚧
- **Fecha:** 2025-12-15
- **Sprint:** Sprint 63/US-38

## 🎯 Objetivo
Implementar dead code elimination (DCE) para eliminar código inalcanzable en el compilador Vela, reduciendo el tamaño del bytecode generado y mejorando el rendimiento al eliminar instrucciones que nunca se ejecutan.

## 🔨 Implementación

### Arquitectura de Dead Code Elimination
El DCE se implementará en el módulo `IROptimizer` dentro de `ir_to_bytecode.rs`, extendiendo las capacidades de optimización existentes con análisis de alcanzabilidad.

### Algoritmo de DCE
1. **Análisis de control flow**: Construir grafo de flujo de control (CFG)
2. **Análisis de alcanzabilidad**: Identificar bloques e instrucciones alcanzables
3. **Marcado de código vivo**: Marcar instrucciones que contribuyen al resultado final
4. **Eliminación**: Remover instrucciones no alcanzables o no utilizadas
5. **Limpieza**: Actualizar referencias y saltos afectados

### Tipos de Dead Code a Eliminar

#### ✅ Código Inalcanzable
```vela
fn example() {
    return 42;  // Retorno temprano
    print("nunca se ejecuta");  // DEAD CODE
}
```

#### ✅ Variables No Utilizadas
```vela
fn example() {
    let unused = calculate();  // Variable asignada pero nunca usada
    return 42;
}
```

#### ✅ Funciones No Llamadas
```vela
fn unused_function() {  // Función nunca llamada
    return 42;
}

fn main() {
    return 0;  // No llama a unused_function
}
```

#### ✅ Código Después de Return/Break/Continue
```vela
fn example() {
    if condition {
        return 42;
        print("dead");  // DEAD CODE
    }
}
```

### Estrategia de Implementación
1. **Análisis de uso**: Tracking de variables y funciones utilizadas
2. **Análisis de flujo**: Detección de caminos de ejecución posibles
3. **Conservative approach**: Solo eliminar código provadamente dead
4. **Side effects**: Preservar código con efectos secundarios importantes

### Casos Edge
- **Funciones exportadas**: No eliminar aunque no se usen localmente
- **Efectos secundarios**: Preservar llamadas con side effects
- **Debug code**: Mantener código de debugging cuando aplicable
- **Entry points**: Preservar puntos de entrada de la aplicación

## ✅ Criterios de Aceptación
- [ ] DCE implementado en `IROptimizer`
- [ ] Eliminación de código inalcanzable después de return/break/continue
- [ ] Eliminación de variables no utilizadas
- [ ] Eliminación de funciones no llamadas (excepto exportadas)
- [ ] Tests unitarios para todos los casos de DCE
- [ ] Benchmarks mostrando reducción del 10-15% en tamaño de bytecode
- [ ] Preservación de código con efectos secundarios

## 📊 Métricas Esperadas
- **Reducción de bytecode**: 10-15% menos instrucciones generadas
- **Mejora de rendimiento**: Menos instrucciones a ejecutar en runtime
- **Tiempo de compilación**: Ligeramente mayor debido al análisis
- **Cobertura de eliminación**: 80% de código dead detectable

## 🔗 Referencias
- **Jira:** [VELA-1184](https://velalang.atlassian.net/browse/VELA-1184)
- **Dependencia:** TASK-171 (Constant Folding)
- **Código:** `src/codegen/ir_to_bytecode.rs::IROptimizer`