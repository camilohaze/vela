# ADR-117G: Arquitectura de Async Iterators en Vela

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Vela necesita soporte para async iterators para manejar flujos de datos infinitos o muy grandes de manera eficiente. Los casos de uso incluyen:

- Procesamiento de streams de datos en tiempo real
- APIs que devuelven grandes cantidades de datos
- Eventos del sistema que llegan continuamente
- Integración con bases de datos que soportan cursores

El problema actual es que Vela no tiene una forma nativa de manejar iteración asíncrona, lo que limita su capacidad para aplicaciones reactivas y de streaming.

## Decisión
Implementar async iterators en Vela siguiendo el patrón de lenguajes funcionales modernos (JavaScript/TypeScript, Python, Kotlin) con las siguientes características:

### Arquitectura General
- **Async Generators**: Funciones `async function*` que pueden `yield` valores
- **Async Iterators**: Objetos que implementan el protocolo async iterator
- **Lazy Evaluation**: Los valores se producen solo cuando se consumen
- **Backpressure**: Control automático de flujo para evitar sobrecarga

### Sintaxis Propuesta
```vela
// Async generator function
async function* getDataStream() -> AsyncIterator<Data> {
    while (true) {
        let data = await fetchNextData()
        yield data
    }
}

// Consumo con for-await
for await (let item of getDataStream()) {
    processItem(item)
}

// Async iterators con métodos funcionales
let stream = getDataStream()
    .map(item => transform(item))
    .filter(item => item.isValid)
    .take(100)
```

### Componentes Técnicos

#### 1. **AsyncIterator<T> Type**
```vela
interface AsyncIterator<T> {
    fn next() -> Promise<Option<T>>
    fn return() -> Promise<void>
    fn throw(error: Error) -> Promise<void>
}
```

#### 2. **Async Generator Functions**
- Sintaxis: `async function* name() -> AsyncIterator<T>`
- Soporte para `yield` y `yield*`
- Manejo automático de estado interno

#### 3. **Stream API**
```vela
class Stream<T> {
    // Transformaciones
    fn map<U>(mapper: (T) -> U) -> Stream<U>
    fn filter(predicate: (T) -> Bool) -> Stream<T>
    fn flatMap<U>(mapper: (T) -> Stream<U>) -> Stream<U>

    // Limitación
    fn take(n: Number) -> Stream<T>
    fn takeWhile(predicate: (T) -> Bool) -> Stream<T>
    fn drop(n: Number) -> Stream<T>

    // Agregación
    fn reduce<U>(initial: U, reducer: (U, T) -> U) -> Promise<U>
    fn collect() -> Promise<List<T>>

    // Consumo
    fn forEach(action: (T) -> void) -> Promise<void>
}
```

#### 4. **Backpressure Mechanism**
- Buffering automático con límites configurables
- Señales de presión hacia arriba en la cadena
- Prevención de memory leaks en streams infinitos

## Consecuencias

### Positivas
- **Mejor ergonomía**: Sintaxis familiar para desarrolladores
- **Performance**: Lazy evaluation evita procesamiento innecesario
- **Composabilidad**: Métodos funcionales permiten chaining
- **Type Safety**: Sistema de tipos fuerte previene errores
- **Backpressure**: Manejo automático de recursos

### Negativas
- **Complejidad**: Implementación más compleja en el compilador
- **Runtime overhead**: Costo adicional para async operations
- **Memory management**: Gestión más compleja del estado

## Alternativas Consideradas

### 1. **Callbacks-based (Rechazada)**
```vela
// Alternativa rechazada
fn getDataStream(callback: (Data) -> void) -> void {
    // Callback hell, difícil de componer
}
```
**Razón**: No composable, callback hell, difícil de testear.

### 2. **Reactive Streams (Rechazada)**
```vela
// Alternativa rechazada
let stream = ReactiveStream.create()
    .subscribe(onNext, onError, onComplete)
```
**Razón**: Más verboso, API más compleja, menos familiar.

### 3. **Pull-based only (Rechazada)**
```vela
// Alternativa rechazada
let iterator = createIterator()
while (let item = await iterator.next()) {
    // Manual iteration only
}
```
**Razón**: Menos expresivo, no soporta operaciones funcionales.

## Implementación

### Fase 1: Core Infrastructure
1. AST nodes para `async function*`
2. Parser para `yield` expressions
3. Codegen para async iterator protocol

### Fase 2: Stream API
1. `AsyncIterator<T>` interface
2. `Stream<T>` class con métodos funcionales
3. Backpressure implementation

### Fase 3: Syntax Sugar
1. `for await` loops
2. `yield*` operator
3. Async comprehension syntax

## Referencias
- **JavaScript/TypeScript**: `async function*`, `for await`
- **Python**: `async def`, `async for`
- **Kotlin**: `Flow<T>`, `flow { }`
- **Rust**: `Stream` trait, `async fn`
- **Jira**: US-25B (VELA-1106)

## Testing Strategy
- Unit tests para cada método de Stream API
- Integration tests para async generators
- Performance benchmarks con streams grandes
- Memory leak tests para streams infinitos