# TASK-059: Implementar Virtual DOM

## 📋 Información General
- **Historia:** VELA-059
- **Epic:** EPIC-05 UI Framework
- **Estado:** En desarrollo
- **Fecha:** 2025-12-03

## 🎯 Objetivo
Implementar un Virtual DOM (VDOM) que sirva como representación intermedia entre los widgets y el DOM real, permitiendo reconciliación eficiente y actualizaciones selectivas.

## 🔨 Especificación Técnica

### Arquitectura del Virtual DOM

#### 1. VDomNode - Nodo Virtual Base
```rust
/// Representación virtual de un elemento del DOM
#[derive(Debug, Clone, PartialEq)]
pub enum VDomNode {
    /// Elemento HTML con atributos y hijos
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<VDomNode>,
        key: Option<String>,
    },

    /// Texto plano
    Text(String),

    /// Fragmento (contenedor sin elemento visual)
    Fragment(Vec<VDomNode>),
}
```

#### 2. VDomTree - Árbol Completo
```rust
/// Árbol completo de Virtual DOM
#[derive(Debug, Clone)]
pub struct VDomTree {
    pub root: VDomNode,
    pub widget_ids: HashMap<String, WidgetId>,
}
```

#### 3. Sistema de Keys
- **Propósito**: Identificación única de widgets para reconciliación eficiente
- **Uso**: Widgets pueden especificar `key` para optimizar re-renders
- **Beneficio**: Evita reconstrucción innecesaria cuando el orden cambia

### Algoritmo de Reconciliación (Diffing)

#### Estrategias de Comparación
1. **Mismo tipo de nodo**: Actualizar atributos/propiedades
2. **Diferente tipo**: Reemplazar nodo completamente
3. **Keys presentes**: Reordenar eficientemente usando keys
4. **Sin keys**: Comparación posicional simple

#### Funciones Core
```rust
/// Comparar dos árboles VDOM y generar diferencias
pub fn diff(old_tree: &VDomTree, new_tree: &VDomTree) -> Vec<VDomPatch>;

/// Tipos de cambios que se pueden aplicar
pub enum VDomPatch {
    /// Insertar nuevo nodo
    Insert { path: VDomPath, node: VDomNode },
    /// Remover nodo existente
    Remove { path: VDomPath },
    /// Actualizar atributos de nodo
    UpdateAttributes { path: VDomPath, attributes: HashMap<String, String> },
    /// Actualizar texto de nodo
    UpdateText { path: VDomPath, text: String },
    /// Reordenar hijos usando keys
    Reorder { path: VDomPath, order: Vec<usize> },
}
```

### Integración con Sistema Reactivo

#### Flujo de Actualización
1. **Signal cambia** → Widget se marca como "dirty"
2. **Re-render** → Widget genera nuevo VDomNode
3. **Diff** → Comparar VDOM anterior vs nuevo
4. **Patch** → Aplicar cambios mínimos al DOM real

#### Optimizaciones
- **Lazy evaluation**: Solo diff cuando es necesario
- **Batching**: Agrupar múltiples cambios en una actualización
- **Memoization**: Evitar re-renders innecesarios

### API Pública

#### Para Widgets
```rust
impl Widget for MyWidget {
    fn build(&self, ctx: &BuildContext) -> VDomNode {
        VDomNode::Element {
            tag: "div".to_string(),
            attributes: HashMap::new(),
            children: vec![
                VDomNode::Text("Hello".to_string()),
            ],
            key: Some(self.id.clone()),
        }
    }
}
```

#### Para Framework Interno
```rust
// Crear árbol VDOM
let vdom = widget.build(&ctx);

// Comparar con versión anterior
let patches = diff(&old_vdom, &vdom);

// Aplicar cambios al DOM real
apply_patches(patches);
```

## ✅ Criterios de Aceptación

### Funcionalidad Core
- [ ] VDomNode representa correctamente elementos, texto y fragmentos
- [ ] Sistema de keys funciona para identificación de widgets
- [ ] Diffing básico (mismo tipo vs diferente tipo)
- [ ] Generación correcta de patches

### Integración
- [ ] Widgets pueden generar VDomNode en su método build()
- [ ] Integración limpia con sistema reactivo
- [ ] Compatibilidad backward con widgets existentes

### Performance
- [ ] Diffing O(n) para árboles típicos
- [ ] Memoria eficiente (no duplicar datos innecesariamente)
- [ ] Batching de actualizaciones múltiples

### Testing
- [ ] Tests unitarios para diffing algorithm
- [ ] Tests de integración con widgets reactivos
- [ ] Tests de performance con árboles grandes

## 🔗 Referencias
- **ADR:** `docs/architecture/ADR-059-virtual-dom.md`
- **Jira:** [VELA-059](https://velalang.atlassian.net/browse/VELA-059)
- **Dependencias:** TASK-058 (Signal Integration)

## 📊 Métricas de Implementación
- **Complejidad:** Media-Alta (algoritmo de diffing)
- **Archivos estimados:** 4-5 (vdom.rs, diff.rs, patch.rs, tests)
- **Tiempo estimado:** 64 horas
- **Riesgos:** Algoritmo de diffing puede ser complejo de optimizar