---
title: "ADR-055: Layout Widgets Architecture"
status: accepted
date: 2024-01-15
deciders: Vela Development Team
consulted: UI Framework Architects
---

## Context

TASK-055 requires implementing fundamental layout widgets: Container, Row, Column, and Stack. These widgets form the foundation of Vela's layout system and must provide:

1. **Flexible layout capabilities** similar to Flutter's layout widgets
2. **Constraint-based sizing** for responsive design
3. **Composition over inheritance** following Rust patterns
4. **Performance optimization** through efficient rendering
5. **Integration with Virtual DOM** system

The layout system needs to handle:
- **Container**: Basic wrapper with padding, margin, alignment
- **Row**: Horizontal layout with main/cross axis alignment
- **Column**: Vertical layout with main/cross axis alignment
- **Stack**: Overlapping widgets with positioning

## Decision

We will implement a constraint-based layout system inspired by Flutter but adapted to Rust's ownership model:

### Layout System Architecture

```rust
/// Core layout constraints
#[derive(Debug, Clone)]
pub struct BoxConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

/// Layout size result
#[derive(Debug, Clone)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// Layout position
#[derive(Debug, Clone)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}
```

### Widget Implementations

#### Container Widget
```rust
#[derive(Debug)]
pub struct Container {
    pub child: Option<Box<dyn Widget>>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub alignment: Option<Alignment>,
    pub decoration: Option<Box<dyn Decoration>>,
}
```

#### Row Widget
```rust
#[derive(Debug)]
pub struct Row {
    pub children: Vec<Box<dyn Widget>>,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
}
```

#### Column Widget
```rust
#[derive(Debug)]
pub struct Column {
    pub children: Vec<Box<dyn Widget>>,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
}
```

#### Stack Widget
```rust
#[derive(Debug)]
pub struct Stack {
    pub children: Vec<PositionedChild>,
    pub alignment: Alignment,
    pub fit: StackFit,
}

#[derive(Debug)]
pub struct PositionedChild {
    pub child: Box<dyn Widget>,
    pub position: Option<Position>,
}
```

### Layout Algorithm

1. **Parent passes constraints** to child widgets
2. **Children perform layout** within constraints
3. **Children return their size** to parent
4. **Parent positions children** based on alignment rules

### Key Design Decisions

#### Constraint-Based Layout
- **Tight constraints**: Fixed size requirements
- **Loose constraints**: Flexible size within bounds
- **Unconstrained**: No size restrictions

#### Alignment System
```rust
#[derive(Debug, Clone)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}
```

#### Performance Optimizations
- **Layout caching** for unchanged subtrees
- **Lazy layout** computation
- **Constraint propagation** optimization
- **Memory-efficient** widget storage

## Consequences

### Positive
- **Flexible layouts**: Support for complex UI arrangements
- **Responsive design**: Constraint-based sizing adapts to screen sizes
- **Performance**: Optimized layout algorithms
- **Developer-friendly**: Intuitive alignment and sizing APIs
- **Composable**: Easy to combine layout widgets

### Negative
- **Complexity**: Layout algorithms can be complex
- **Performance cost**: Layout computation overhead
- **Learning curve**: Understanding constraint system
- **Debugging**: Layout issues can be hard to diagnose

### Alternatives Considered

1. **CSS-Style Layout**
   - ✅ Familiar to web developers
   - ❌ Complex box model calculations
   - ❌ Not optimized for programmatic layout

2. **Absolute Positioning Only**
   - ✅ Simple implementation
   - ❌ Hard to create responsive layouts
   - ❌ Manual positioning calculations

3. **Grid-Based Layout**
   - ✅ Powerful for complex layouts
   - ❌ Overkill for simple cases
   - ❌ Complex API for basic layouts

## Implementation

### Phase 1: Core Layout System
- Implement `BoxConstraints`, `Size`, `Offset`
- Create layout traits and algorithms
- Basic constraint propagation

### Phase 2: Container Widget
- Simple wrapper with padding/margin
- Alignment support
- Decoration system foundation

### Phase 3: Flex Layout (Row/Column)
- Main/cross axis alignment
- Flexible sizing
- Baseline alignment support

### Phase 4: Stack Layout
- Positioned children
- Z-index management
- Overflow handling

### Phase 5: Advanced Features
- Intrinsic sizing
- Layout debugging tools
- Performance profiling

## Testing

Comprehensive test coverage for:
- **Layout algorithms**: Constraint propagation, sizing calculations
- **Widget integration**: Proper VDOM generation
- **Alignment**: All alignment combinations
- **Performance**: Layout caching and optimization
- **Edge cases**: Zero-sized widgets, overflow conditions

## References

- TASK-055: Implementar widgets de layout (Container, Row, Column, Stack)
- Flutter layout system
- CSS Flexbox specification
- React Native layout system
- TASK-054: Widget base class (completed)</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-055-layout-widgets.md