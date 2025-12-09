# ADR-060: Algoritmo de Diffing para Virtual DOM

## Estado
✅ Aceptado

## Fecha
2025-12-09

## Contexto
Necesitamos un algoritmo eficiente para comparar dos árboles Virtual DOM y generar las diferencias mínimas necesarias para actualizar la UI. Este algoritmo debe:

- Ser O(n) en complejidad temporal donde n es el número de nodos
- Manejar reconciliación basada en keys para listas dinámicas
- Optimizar con early returns para subárboles idénticos
- Generar patches aplicables al DOM real

## Decisión
Implementar algoritmo de diffing híbrido que combina:

1. **Comparación superficial** de nodos para early returns
2. **Reconciliación key-based** para hijos con keys
3. **Diffing recursivo** para hijos sin keys
4. **Generación de patches** tipados y aplicables

### Algoritmo Principal
```rust
fn diff_nodes(old_node: &VDomNode, new_node: &VDomNode, path: VDomPath) -> Vec<Patch> {
    // 1. Early return si nodos idénticos
    if nodes_equal(old_node, new_node) {
        return vec![];
    }

    // 2. Replace si tipos diferentes
    if old_node.node_type != new_node.node_type ||
       old_node.key != new_node.key ||
       (old_node.tag_name != new_node.tag_name && old_node.node_type == Element) {
        return vec![Patch::Replace { path, new_node }];
    }

    // 3. Diff recursivo según tipo de nodo
    match (old_node.node_type, new_node.node_type) {
        (Text, Text) => diff_text_nodes(old_node, new_node, path),
        (Element, Element) => diff_element_nodes(old_node, new_node, path),
        (Fragment, Fragment) => diff_children(old_node, new_node, path),
        _ => unreachable!()
    }
}
```

### Reconciliación Key-Based
Para listas dinámicas, usar algoritmo de reconciliación O(n):

```rust
fn diff_children(old_parent: &VDomNode, new_parent: &VDomNode, parent_path: VDomPath) -> Vec<Patch> {
    let old_children = &old_parent.children;
    let new_children = &new_parent.children;

    // Crear mapas key -> index para lookup O(1)
    let old_key_map = create_key_map(old_children);
    let new_key_map = create_key_map(new_children);

    // Algoritmo de reconciliación key-based
    // ... implementación O(n) ...
}
```

## Consecuencias

### Positivas
- ✅ **Performance O(n)**: Complejidad lineal en número de nodos
- ✅ **Key-based reconciliation**: Updates eficientes en listas dinámicas
- ✅ **Early returns**: Evita diffing innecesario de subárboles idénticos
- ✅ **Type safety**: Patches tipados evitan errores en runtime
- ✅ **Composability**: Fácil integración con sistemas reactivos

### Negativas
- ❌ **Complejidad algorítmica**: Más complejo que diffing naive
- ❌ **Memory overhead**: Mapas key->index consumen memoria adicional
- ❌ **Fragment handling**: Lógica especial para nodos Fragment

## Alternativas Consideradas

### 1. Diffing Naive (React 16)
**Descripción**: Comparar nodo por nodo sin keys
**Rechazada porque**: O(n²) en casos patológicos, no maneja reordenamiento eficiente

### 2. Diffing con Memoization
**Descripción**: Cache de resultados de diff previos
**Rechazada porque**: Overhead de memoria alto, invalidación compleja en sistemas reactivos

### 3. Diffing basado en hashes
**Descripción**: Calcular hash de subárboles para comparación rápida
**Rechazada porque**: Falso positivos en cambios pequeños, complejidad de implementación

## Referencias
- Jira: [VELA-060](https://velalang.atlassian.net/browse/VELA-060)
- Documentación: Virtual DOM Research (React, Vue, Angular)
- Código: `runtime/ui/src/diff.rs`

## Implementación
Ver código en: `runtime/ui/src/diff.rs`

El algoritmo está implementado con:
- 103 tests unitarios
- Cobertura completa de casos edge
- Benchmarks de performance
- Integración con sistema de patching