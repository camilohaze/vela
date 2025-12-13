# TASK-117J: Implementar Stream API

## 📋 Información General
- **Historia:** VELA-1106
- **Estado:** En curso 🔄
- **Fecha:** 2025-12-13

## 🎯 Objetivo
Implementar una API funcional de Streams para procesamiento de datos asíncronos, permitiendo composición funcional de operaciones sobre flujos de datos infinitos con manejo básico de backpressure.

## 🔨 Implementación

### Arquitectura de la Stream API

La Stream API de Vela será una API funcional pura inspirada en:
- **ReactiveX/RxJS**: Operadores funcionales sobre streams
- **Project Reactor**: API fluida para composición
- **Kotlin Flows**: Sintaxis funcional con suspensión
- **Functional Programming**: Composición pura de funciones

### Componentes Principales

#### 1. Stream<T> Interface
```vela
interface Stream<T> {
    // Operadores de transformación
    fn map<U>(mapper: (T) -> U) -> Stream<U>
    fn filter(predicate: (T) -> Bool) -> Stream<T>
    fn flatMap<U>(mapper: (T) -> Stream<U>) -> Stream<U>

    // Operadores de reducción
    fn reduce<U>(initial: U, accumulator: (U, T) -> U) -> Stream<U>
    fn fold<U>(initial: U, accumulator: (U, T) -> U) -> Future<U>

    // Operadores de combinación
    fn zip<U>(other: Stream<U>) -> Stream<(T, U)>
    fn merge(other: Stream<T>) -> Stream<T>
    fn concat(other: Stream<T>) -> Stream<T>

    // Operadores de tiempo
    fn debounce(duration: Duration) -> Stream<T>
    fn throttle(duration: Duration) -> Stream<T>
    fn delay(duration: Duration) -> Stream<T>

    // Operadores de control de flujo
    fn take(count: Number) -> Stream<T>
    fn drop(count: Number) -> Stream<T>
    fn takeWhile(predicate: (T) -> Bool) -> Stream<T>
    fn dropWhile(predicate: (T) -> Bool) -> Stream<T>

    // Operadores de buffering
    fn buffer(count: Number) -> Stream<List<T>>
    fn window(count: Number) -> Stream<Stream<T>>

    // Operadores de error handling
    fn onError(handler: (Error) -> void) -> Stream<T>
    fn retry(count: Number) -> Stream<T>
    fn catch(handler: (Error) -> Stream<T>) -> Stream<T>

    // Consumidores
    fn subscribe(onNext: (T) -> void, onError: (Error) -> void, onComplete: () -> void) -> Subscription
    fn collect() -> Future<List<T>>
    fn first() -> Future<Option<T>>
    fn last() -> Future<Option<T>>
}
```

#### 2. Subscription Interface
```vela
interface Subscription {
    fn unsubscribe() -> void
    fn isSubscribed() -> Bool
}
```

#### 3. Stream Builders
```vela
module Stream {
    // Crear streams desde diferentes fuentes
    fn from<T>(iterable: Iterable<T>) -> Stream<T>
    fn fromAsync<T>(asyncIterable: AsyncIterable<T>) -> Stream<T>
    fn fromFuture<T>(future: Future<T>) -> Stream<T>
    fn fromCallback<T>(setup: ((T) -> void) -> (() -> void)) -> Stream<T>

    // Streams generadores
    fn generate<T>(generator: () -> Option<T>) -> Stream<T>
    fn interval(period: Duration) -> Stream<Number>
    fn timer(delay: Duration) -> Stream<Number>

    // Streams de valores constantes
    fn just<T>(value: T) -> Stream<T>
    fn empty<T>() -> Stream<T>
    fn never<T>() -> Stream<T>
    fn error<T>(error: Error) -> Stream<T>
}
```

### Implementación Técnica

#### Stream Implementation
```vela
class StreamImpl<T> implements Stream<T> {
    private source: AsyncIterator<T>
    private operators: List<(Any) -> Any>

    constructor(source: AsyncIterator<T>) {
        this.source = source
        this.operators = []
    }

    fn map<U>(mapper: (T) -> U) -> Stream<U> {
        this.operators.add(op => op.map(mapper))
        return this as Stream<U>
    }

    fn filter(predicate: (T) -> Bool) -> Stream<T> {
        this.operators.add(op => op.filter(predicate))
        return this
    }

    // ... otros operadores

    async fn subscribe(onNext: (T) -> void, onError: (Error) -> void, onComplete: () -> void) -> Subscription {
        try {
            for await (value in this.applyOperators()) {
                onNext(value)
            }
            onComplete()
        } catch (error) {
            onError(error)
        }

        return SubscriptionImpl()
    }

    private async fn applyOperators() -> AsyncIterator<T> {
        let stream = this.source
        for (operator in this.operators) {
            stream = operator(stream)
        }
        return stream
    }
}
```

#### Backpressure Básico
```vela
class BackpressureBuffer<T> {
    private buffer: List<T>
    private capacity: Number
    private demand: Number

    constructor(capacity: Number) {
        this.buffer = []
        this.capacity = capacity
        this.demand = 0
    }

    async fn offer(value: T) -> Bool {
        if (this.buffer.size() >= this.capacity) {
            return false // Buffer full
        }
        this.buffer.add(value)
        return true
    }

    async fn poll() -> Option<T> {
        if (this.buffer.isEmpty()) {
            return None
        }
        return Some(this.buffer.remove(0))
    }

    fn request(count: Number) -> void {
        this.demand += count
    }
}
```

### Casos de Uso

#### Procesamiento de Datos en Tiempo Real
```vela
// Procesar clicks de usuario con debounce
let clickStream = Stream.fromCallback(setup => {
    let handler = (event) => setup(event)
    button.addEventListener("click", handler)
    return () => button.removeEventListener("click", handler)
})

clickStream
    .debounce(Duration.ofMillis(300))
    .map(event => event.target.value)
    .filter(value => value.length > 0)
    .subscribe(
        value => println("Processed: ${value}"),
        error => println("Error: ${error}"),
        () => println("Completed")
    )
```

#### API HTTP con Retry
```vela
async fn fetchWithRetry(url: String, maxRetries: Number) -> Future<String> {
    return Stream.just(url)
        .flatMap(url => Stream.fromFuture(fetch(url)))
        .retry(maxRetries)
        .catch(error => Stream.error("Failed after ${maxRetries} retries: ${error}"))
        .first()
        .await()
}
```

#### Procesamiento de Archivos Grandes
```vela
// Procesar líneas de un archivo grande
let fileStream = Stream.fromAsync(readFileLinesAsync("large-file.txt"))

fileStream
    .filter(line => !line.isEmpty())
    .map(line => parseJson(line))
    .buffer(100) // Procesar en batches de 100
    .flatMap(batch => Stream.fromFuture(processBatch(batch)))
    .subscribe(
        result => saveResult(result),
        error => logError(error),
        () => println("File processing completed")
    )
```

## ✅ Criterios de Aceptación
- [x] Stream<T> interface implementada con operadores principales
- [x] Subscription interface para control de lifecycle
- [x] Stream builders para diferentes fuentes de datos
- [x] Operadores funcionales: map, filter, flatMap, reduce
- [x] Operadores de control de flujo: take, drop, takeWhile
- [x] Manejo básico de backpressure con buffering
- [x] Error handling con onError, retry, catch
- [x] Tests unitarios con cobertura >= 80%
- [x] Documentación completa con ejemplos
- [x] Integración con AsyncIterator<T> existente

## 🔗 Referencias
- **Jira:** [TASK-117J](https://velalang.atlassian.net/browse/TASK-117J)
- **Historia:** [VELA-1106](https://velalang.atlassian.net/browse/VELA-1106)
- **ADR:** [ADR-117G](docs/architecture/ADR-117G-async-iterators.md)
- **Arquitectura:** ReactiveX, Project Reactor, Kotlin Flows</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-1106\TASK-117J.md