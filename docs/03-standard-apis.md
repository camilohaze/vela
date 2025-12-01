# 3. Especificación de APIs Estándar de Vela

## 3.1 Biblioteca Core (vela.core)

### 3.1.1 Primitivos y Tipos Básicos

```vela
module vela.core;

// Int
interface Int {
  fn toString(): String;
  fn toFloat(): Float;
  fn abs(): Int;
  fn pow(exponent: Int): Int;
  fn clamp(min: Int, max: Int): Int;
  
  // Bitwise operations
  fn bitwiseAnd(other: Int): Int;
  fn bitwiseOr(other: Int): Int;
  fn bitwiseXor(other: Int): Int;
  fn shiftLeft(bits: Int): Int;
  fn shiftRight(bits: Int): Int;
}

// Float
interface Float {
  fn toString(): String;
  fn toInt(): Int;
  fn round(): Float;
  fn floor(): Float;
  fn ceil(): Float;
  fn abs(): Float;
  fn sqrt(): Float;
  fn pow(exponent: Float): Float;
  fn clamp(min: Float, max: Float): Float;
  
  fn isNaN(): Bool;
  fn isInfinite(): Bool;
  fn isFinite(): Bool;
}

// String
interface String {
  fn length: Int;
  
  fn charAt(index: Int): Char;
  fn substring(start: Int, end: Int?): String;
  fn split(separator: String): List<String>;
  fn trim(): String;
  fn trimStart(): String;
  fn trimEnd(): String;
  fn toUpperCase(): String;
  fn toLowerCase(): String;
  fn replace(search: String, replacement: String): String;
  fn replaceAll(search: String, replacement: String): String;
  fn contains(substring: String): Bool;
  fn startsWith(prefix: String): Bool;
  fn endsWith(suffix: String): Bool;
  fn indexOf(substring: String): Option<Int>;
  fn lastIndexOf(substring: String): Option<Int>;
  fn matches(pattern: Regex): Bool;
  fn repeat(times: Int): String;
  fn padStart(length: Int, fill: String): String;
  fn padEnd(length: Int, fill: String): String;
}

// Bool
interface Bool {
  fn toString(): String;
  fn not(): Bool;
}

// Char
interface Char {
  fn toString(): String;
  fn toInt(): Int;
  fn isDigit(): Bool;
  fn isLetter(): Bool;
  fn isWhitespace(): Bool;
  fn isUpperCase(): Bool;
  fn isLowerCase(): Bool;
  fn toUpperCase(): Char;
  fn toLowerCase(): Char;
}
```

---

### 3.1.2 Colecciones

#### List<T> - Functional Methods (NO loops)

```vela
public class List<T> {
  // Properties
  public fn length: Int;
  public fn isEmpty: Bool;
  
  // Constructors
  public List();
  public List(items: T[]);
  
  // Access
  public fn get(index: Int): T;
  public fn set(index: Int, value: T): void;
  public fn first(): Option<T>;
  public fn last(): Option<T>;
  public fn at(index: Int): Option<T>;
  
  // Modification (retorna nueva lista - inmutable)
  public fn push(item: T): List<T>;
  public fn pop(): (Option<T>, List<T>);
  public fn insert(index: Int, item: T): List<T>;
  public fn remove(index: Int): (T, List<T>);
  
  // Query
  public fn contains(item: T): Bool;
  public fn indexOf(item: T): Option<Int>;
  public fn lastIndexOf(item: T): Option<Int>;
  
  // Transformation (inmutable - NO loops)
  public fn map<U>(fn: (T) => U): List<U>;
  public fn filter(predicate: (T) => Bool): List<T>;
  public fn reduce<U>(initial: U, fn: (U, T) => U): U;
  public fn flatMap<U>(fn: (T) => List<U>): List<U>;
  public fn sort(compareFn: (T, T) => Int): List<T>;
  public fn reverse(): List<T>;
  public fn slice(start: Int, end: Option<Int>): List<T>;
  public fn concat(other: List<T>): List<T>;
  public fn zip<U>(other: List<U>): List<(T, U)>;
  
  // Aggregation (funcional - NO loops)
  public fn forEach(fn: (T) => void): void;
  public fn every(predicate: (T) => Bool): Bool;
  public fn some(predicate: (T) => Bool): Bool;
  public fn find(predicate: (T) => Bool): Option<T>;
  public fn findIndex(predicate: (T) => Bool): Option<Int>;
  public fn take(n: Int): List<T>;
  public fn drop(n: Int): List<T>;
  public fn takeWhile(predicate: (T) => Bool): List<T>;
  public fn dropWhile(predicate: (T) => Bool): List<T>;
  
  // Conversion
  public fn toSet(): Set<T>;
  public fn toArray(): T[];
  public fn join(separator: String): String;
}
```

**Nota**: Todas las transformaciones retornan nuevas listas (inmutabilidad). NO usar loops - usar métodos funcionales.

#### Set<T>

```vela
public class Set<T> {
  public fn size: Int;
  public fn isEmpty: Bool;
  
  public Set();
  public Set(items: List<T>);
  
  public fn add(item: T): void;
  public fn remove(item: T): Bool;
  public fn contains(item: T): Bool;
  public fn clear(): void;
  
  public fn union(other: Set<T>): Set<T>;
  public fn intersection(other: Set<T>): Set<T>;
  public fn difference(other: Set<T>): Set<T>;
  public fn isSubsetOf(other: Set<T>): Bool;
  public fn isSupersetOf(other: Set<T>): Bool;
  
  public fn toList(): List<T>;
  public fn forEach(fn: (T) => void): void;
  public fn filter(predicate: (T) => Bool): Set<T>;
  public fn map<U>(fn: (T) => U): Set<U>;
}
```

#### Dict<K, V>

```vela
public class Dict<K, V> {
  public fn size: Int;
  public fn isEmpty: Bool;
  
  public Dict();
  public Dict(entries: List<(K, V)>);
  
  public fn get(key: K): Option<V>;  // NO null - retorna Option<V>
  public fn set(key: K, value: V): Dict<K, V>;  // Retorna nuevo Dict (inmutable)
  public fn has(key: K): Bool;
  public fn remove(key: K): Dict<K, V>;  // Retorna nuevo Dict
  
  public fn keys(): List<K>;
  public fn values(): List<V>;
  public fn entries(): List<(K, V)>;
  
  // Functional methods (NO loops)
  public fn forEach(fn: (K, V) => void): void;
  public fn map<U>(fn: (K, V) => U): Dict<K, U>;
  public fn filter(predicate: (K, V) => Bool): Dict<K, V>;
  
  public fn merge(other: Dict<K, V>): Dict<K, V>;
}
```

#### Queue<T>

```vela
public class Queue<T> {
  public fn size: Int;
  public fn isEmpty: Bool;
  
  public Queue();
  
  public fn enqueue(item: T): void;
  public fn dequeue(): Option<T>;
  public fn peek(): Option<T>;
  public fn clear(): void;
  
  public fn toList(): List<T>;
}
```

#### Stack<T>

```vela
public class Stack<T> {
  public fn size: Int;
  public fn isEmpty: Bool;
  
  public Stack();
  
  public fn push(item: T): void;
  public fn pop(): Option<T>;
  public fn peek(): Option<T>;
  public fn clear(): void;
  
  public fn toList(): List<T>;
}
```

---

### 3.1.3 Option<T> y Result<T, E> (NO null)

```vela
// Reemplazo de null - null-safety garantizado
public enum Option<T> {
  Some(T),
  None  // NO null keyword
}

extension Option<T> {
  public fn isSome(): Bool;
  public fn isNone(): Bool;
  
  public fn unwrap(): T;  // Panic if None
  public fn unwrapOr(default: T): T;
  public fn unwrapOrElse(fn: () => T): T;
  
  public fn map<U>(fn: (T) => U): Option<U>;
  public fn flatMap<U>(fn: (T) => Option<U>): Option<U>;
  public fn filter(predicate: (T) => Bool): Option<T>;
  
  public fn match<U>(onSome: (T) => U, onNone: () => U): U;
  
  // Operadores
  public fn or(other: Option<T>): Option<T>;
  public fn and<U>(other: Option<U>): Option<U>;
}

public enum Result<T, E> {
  Ok(T),
  Err(E)
}

extension Result<T, E> {
  public fn isOk(): Bool;
  public fn isErr(): Bool;
  
  public fn unwrap(): T;  // Panic if Err
  public fn unwrapOr(default: T): T;
  public fn unwrapErr(): E;  // Panic if Ok
  
  public fn map<U>(fn: (T) => U): Result<U, E>;
  public fn mapErr<F>(fn: (E) => F): Result<T, F>;
  public fn flatMap<U>(fn: (T) => Result<U, E>): Result<U, E>;
  
  public fn match<U>(onOk: (T) => U, onErr: (E) => U): U;
}
```

**Importante**: NO existe `null` en Vela. Usar `Option.None` para valores ausentes.

---

## 3.2 APIs de Signals Reactivos (vela.reactive)

### 3.2.1 Signal<T>

```vela
module vela.reactive;

public class Signal<T> {
  // Constructors
  public static fn create(initialValue: T): Signal<T>;
  
  // Properties
  public fn value: T {
    get => this.getValue();
    set(newValue) => this.setValue(newValue);
  }
  
  // Methods
  public fn getValue(): T;
  public fn setValue(newValue: T): void;
  public fn update(updater: (T) => T): void;
  
  // Subscriptions
  public fn subscribe(listener: (T) => void): Subscription;
  
  // Operators
  public fn map<U>(mapper: (T) => U): Computed<U>;
  public fn filter(predicate: (T) => Bool): Computed<T?>;
  
  // Debugging
  public fn debug(label: String): Signal<T>;
}

public interface Subscription {
  fn unsubscribe(): void;
}
```

### 3.2.2 Computed<T>

```vela
public class Computed<T> {
  // Constructors
  public static fn create<T>(compute: () => T): Computed<T>;
  
  // Properties
  public fn value: T {
    get => this.getValue();
  }
  
  // Methods
  public fn getValue(): T;
  public fn recompute(): void;
  
  // Dependencies
  public fn getDependencies(): List<Signal<any>>;
  
  // Operators
  public fn map<U>(mapper: (T) => U): Computed<U>;
  
  // Debugging
  public fn debug(label: String): Computed<T>;
}
```

### 3.2.3 Effect

```vela
public class Effect {
  // Constructors
  public static fn create(effectFn: () => void): Effect;
  
  // Control
  public fn pause(): void;
  public fn resume(): void;
  public fn stop(): void;
  public fn isActive(): Bool;
  
  // Manual trigger
  public fn trigger(): void;
}
```

### 3.2.4 Watch

```vela
public fn watch<T>(
  source: Signal<T>,
  callback: (newValue: T, oldValue: T) => void,
  options: WatchOptions?
): Subscription;

public class WatchOptions {
  public immediate: Bool = false;
  public deep: Bool = false;
}
```

### 3.2.5 Memo

```vela
public fn memo<T>(
  compute: () => T,
  equals: (a: T, b: T) => Bool = defaultEquals
): Computed<T>;
```

### 3.2.6 Batch

```vela
public fn batch(fn: () => void): void;

// Example
batch(() => {
  signal1.value = 10;
  signal2.value = 20;
  signal3.value = 30;
  // Solo 1 re-render en lugar de 3
});
```

### 3.2.7 Reactive Context

```vela
public class ReactiveContext {
  public static fn current(): ReactiveContext?;
  public static fn run<T>(fn: () => T): T;
  
  public fn track(signal: Signal<any>): void;
  public fn trigger(): void;
}
```

---

## 3.3 APIs de Concurrencia (vela.async)

### 3.3.1 Future<T> y Promise<T>

```vela
module vela.async;

public interface Future<T> {
  fn await(): T;
  fn then<U>(onSuccess: (T) => U): Future<U>;
  fn catch<E>(onError: (E) => T): Future<T>;
  fn finally(onFinally: () => void): Future<T>;
}

public class Promise<T> {
  public static fn resolve<T>(value: T): Promise<T>;
  public static fn reject<T>(error: Error): Promise<T>;
  public static fn all<T>(promises: List<Promise<T>>): Promise<List<T>>;
  public static fn race<T>(promises: List<Promise<T>>): Promise<T>;
  public static fn any<T>(promises: List<Promise<T>>): Promise<T>;
  
  public fn then<U>(onSuccess: (T) => U): Promise<U>;
  public fn catch(onError: (Error) => T): Promise<T>;
  public fn finally(onFinally: () => void): Promise<T>;
}
```

### 3.3.2 Actor<T>

```vela
public abstract class Actor<T> {
  // Message handling
  protected abstract fn receive(message: T): void;
  
  // Lifecycle
  protected fn onStart(): void {}
  protected fn onStop(): void {}
  protected fn onError(error: Error): void {}
  
  // Control
  public fn send(message: T): void;
  public async fn ask<U>(message: T): Future<U>;
  public fn stop(): void;
  
  // State
  public fn isRunning(): Bool;
  public fn getMailboxSize(): Int;
}

// Example usage - state mutable dentro del actor
actor Counter {
  state count: Int = 0;  // Mutable con 'state'
  
  on Increment() {
    this.count = this.count + 1;
  }
  
  on Decrement() {
    this.count = this.count - 1;
  }
  
  on GetCount(): Int {
    return this.count;
  }
}
```

### 3.3.3 Worker

```vela
public class Worker {
  public static fn spawn<T>(task: () => T): Worker;
  public static fn spawnAsync<T>(task: async () => T): Worker;
  
  public fn await<T>(): T;
  public fn isFinished(): Bool;
  public fn cancel(): void;
}

// Example
let worker = Worker.spawn(() => {
  // Computación pesada
  return heavyComputation();
});

let result = worker.await();
```

### 3.3.4 Channel<T>

```vela
public class Channel<T> {
  public static fn create<T>(capacity: Option<Int>): Channel<T>;
  
  public async fn send(value: T): void;
  public async fn receive(): T;
  public fn tryReceive(): Option<T>;
  public fn close(): void;
  public fn isClosed(): Bool;
}

// Example - NO loops, usar recursión
channel = Channel.create<Int>(Option.Some(10));

// Producer usando recursión funcional
async fn producer(n: Int): void {
  if (n >= 100) {
    channel.close();
    return;
  }
  await channel.send(n);
  await producer(n + 1);  // Recursión en lugar de loop
}

// Consumer usando recursión
async fn consumer(): void {
  value = await channel.receive();
  print(value);
  if (!channel.isClosed()) {
    await consumer();  // Recursión
  }
}
```

### 3.3.5 Task

```vela
public class Task<T> {
  public static fn run<T>(fn: async () => T): Task<T>;
  public static fn delay(milliseconds: Int): Task<void>;
  public static fn whenAll<T>(tasks: List<Task<T>>): Task<List<T>>;
  public static fn whenAny<T>(tasks: List<Task<T>>): Task<T>;
  
  public fn await(): T;
  public fn cancel(): void;
  public fn isCancelled(): Bool;
  public fn isCompleted(): Bool;
}
```

---

## 3.4 APIs de UI Declarativa (vela.ui)

### 3.4.1 Widget Base

```vela
module vela.ui;

public abstract class Widget {
  // Lifecycle
  protected fn onMount(): void {}
  protected fn onUpdate(): void {}
  protected fn onDestroy(): void {}
  
  // Build
  public abstract fn build(): Widget;
  
  // State
  protected fn setState(updater: () => void): void;
  
  // Context
  protected fn getContext<T>(key: ContextKey<T>): T?;
}
```

### 3.4.2 Layout Widgets

```vela
// Container
public fn Container(
  child: Widget?,
  padding: EdgeInsets?,
  margin: EdgeInsets?,
  width: Float?,
  height: Float?,
  color: Color?,
  border: Border?,
  borderRadius: BorderRadius?,
  alignment: Alignment?
): Widget;

// Row (horizontal layout)
public fn Row(
  children: List<Widget>,
  spacing: Float?,
  mainAxisAlignment: MainAxisAlignment?,
  crossAxisAlignment: CrossAxisAlignment?
): Widget;

// Column (vertical layout)
public fn Column(
  children: List<Widget>,
  spacing: Float?,
  mainAxisAlignment: MainAxisAlignment?,
  crossAxisAlignment: CrossAxisAlignment?
): Widget;

// Stack (z-axis stacking)
public fn Stack(
  children: List<Widget>,
  alignment: Alignment?
): Widget;

// Flex (flexible layout)
public fn Flex(
  direction: FlexDirection,
  children: List<Widget>,
  spacing: Float?
): Widget;

// Grid
public fn Grid(
  children: List<Widget>,
  columns: Int,
  rows: Int?,
  columnGap: Float?,
  rowGap: Float?
): Widget;
```

### 3.4.3 Input Widgets

```vela
// Button
public fn Button(
  label: String?,
  child: Widget?,
  onClick: () => void,
  disabled: Bool?,
  variant: ButtonVariant?
): Widget;

// TextField
public fn TextField(
  value: Signal<String>,
  placeholder: String?,
  multiline: Bool?,
  maxLength: Int?,
  onChanged: (String) => void?,
  onEnter: () => void?
): Widget;

// Checkbox
public fn Checkbox(
  value: Signal<Bool>,
  label: String?,
  onChange: (Bool) => void?
): Widget;

// Radio
public fn Radio<T>(
  groupValue: Signal<T>,
  value: T,
  label: String?,
  onChange: (T) => void?
): Widget;

// Slider
public fn Slider(
  value: Signal<Float>,
  min: Float,
  max: Float,
  step: Float?,
  onChange: (Float) => void?
): Widget;

// Dropdown
public fn Dropdown<T>(
  value: Signal<T>,
  items: List<DropdownItem<T>>,
  onChange: (T) => void?
): Widget;
```

### 3.4.4 Display Widgets

```vela
// Text
public fn Text(
  text: String,
  style: TextStyle?
): Widget;

// Image
public fn Image(
  src: String,
  width: Float?,
  height: Float?,
  fit: ImageFit?
): Widget;

// Icon
public fn Icon(
  icon: IconData,
  size: Float?,
  color: Color?
): Widget;
```

### 3.4.5 Estilos

```vela
public class TextStyle {
  public fontSize: Float?;
  public fontWeight: FontWeight?;
  public fontStyle: FontStyle?;
  public color: Color?;
  public decoration: TextDecoration?;
  public letterSpacing: Float?;
  public lineHeight: Float?;
}

public enum FontWeight {
  Thin, ExtraLight, Light, Normal, Medium, SemiBold, Bold, ExtraBold, Black
}

public class Color {
  public static fn rgb(r: Int, g: Int, b: Int): Color;
  public static fn rgba(r: Int, g: Int, b: Int, a: Float): Color;
  public static fn hex(hexString: String): Color;
  
  // Predefined colors
  public static fn red: Color;
  public static fn green: Color;
  public static fn blue: Color;
  // ... más colores
}

public class EdgeInsets {
  public static fn all(value: Float): EdgeInsets;
  public static fn symmetric(vertical: Float?, horizontal: Float?): EdgeInsets;
  public static fn only(top: Float?, right: Float?, bottom: Float?, left: Float?): EdgeInsets;
}
```

---

## 3.5 APIs de I/O y Sistema (vela.io)

### 3.5.1 File System

```vela
module vela.io;

public class File {
  public static fn open(path: String, mode: FileMode): Result<File, IoError>;
  
  public fn read(): Result<String, IoError>;
  public fn readBytes(): Result<List<Byte>, IoError>;
  public async fn readAsync(): Result<String, IoError>;
  
  public fn write(content: String): Result<void, IoError>;
  public fn writeBytes(bytes: List<Byte>): Result<void, IoError>;
  public async fn writeAsync(content: String): Result<void, IoError>;
  
  public fn append(content: String): Result<void, IoError>;
  public fn close(): void;
  
  public fn size(): Result<Int, IoError>;
  public fn exists(): Bool;
  public fn delete(): Result<void, IoError>;
  public fn rename(newPath: String): Result<void, IoError>;
}

public class Directory {
  public static fn create(path: String): Result<Directory, IoError>;
  public static fn open(path: String): Result<Directory, IoError>;
  
  public fn list(): Result<List<String>, IoError>;
  public fn listFiles(): Result<List<File>, IoError>;
  public fn listDirectories(): Result<List<Directory>, IoError>;
  
  public fn exists(): Bool;
  public fn delete(recursive: Bool): Result<void, IoError>;
}

public class Path {
  public static fn join(parts: String...): String;
  public static fn normalize(path: String): String;
  public static fn basename(path: String): String;
  public static fn dirname(path: String): String;
  public static fn extension(path: String): String;
  public static fn isAbsolute(path: String): Bool;
}
```

### 3.5.2 Networking (HTTP)

```vela
module vela.net;

public class HttpClient {
  public HttpClient(config: HttpConfig?);
  
  public async fn get(url: String, options: RequestOptions?): Result<HttpResponse, HttpError>;
  public async fn post(url: String, body: any, options: RequestOptions?): Result<HttpResponse, HttpError>;
  public async fn put(url: String, body: any, options: RequestOptions?): Result<HttpResponse, HttpError>;
  public async fn delete(url: String, options: RequestOptions?): Result<HttpResponse, HttpError>;
  public async fn patch(url: String, body: any, options: RequestOptions?): Result<HttpResponse, HttpError>;
  
  public async fn request(request: HttpRequest): Result<HttpResponse, HttpError>;
}

public class HttpResponse {
  public fn status: Int;
  public fn statusText: String;
  public fn headers: Dict<String, String>;
  
  public async fn text(): String;
  public async fn json<T>(): T;
  public async fn bytes(): List<Byte>;
}

public class HttpRequest {
  public url: String;
  public method: HttpMethod;
  public headers: Dict<String, String>;
  public body: any?;
  public timeout: Int?;
}

public enum HttpMethod {
  GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
}
```

### 3.5.3 WebSocket

```vela
public class WebSocket {
  public static fn connect(url: String): Result<WebSocket, WebSocketError>;
  
  public fn send(message: String): void;
  public fn sendBytes(data: List<Byte>): void;
  public fn close(): void;
  
  public fn onMessage(handler: (String) => void): Subscription;
  public fn onClose(handler: (CloseEvent) => void): Subscription;
  public fn onError(handler: (Error) => void): Subscription;
  
  public fn isConnected(): Bool;
}
```

---

## 3.6 APIs de Serialización (vela.encoding)

### 3.6.1 JSON

```vela
module vela.encoding;

public class Json {
  public static fn encode<T>(value: T): Result<String, JsonError>;
  public static fn decode<T>(json: String): Result<T, JsonError>;
  
  public static fn parse(json: String): Result<JsonValue, JsonError>;
  public static fn stringify(value: JsonValue, indent: Int?): String;
}

public enum JsonValue {
  Null,
  Bool(Bool),
  Number(Float),
  String(String),
  Array(List<JsonValue>),
  Object(Dict<String, JsonValue>)
}
```

### 3.6.2 Binary

```vela
public class Binary {
  public static fn encode<T>(value: T): List<Byte>;
  public static fn decode<T>(bytes: List<Byte>): Result<T, BinaryError>;
}
```

---

## 3.7 APIs de Testing (vela.test)

```vela
module vela.test;

public fn describe(name: String, tests: () => void): void;
public fn it(name: String, test: () => void): void;
public fn test(name: String, test: () => void): void;

public fn expect<T>(actual: T): Assertion<T>;

public class Assertion<T> {
  public fn toBe(expected: T): void;
  public fn toEqual(expected: T): void;
  public fn toBeNull(): void;
  public fn toBeTrue(): void;
  public fn toBeFalse(): void;
  public fn toContain(item: any): void;
  public fn toThrow(): void;
  public fn toBeGreaterThan(value: T): void;
  public fn toBeLessThan(value: T): void;
}

public fn beforeEach(fn: () => void): void;
public fn afterEach(fn: () => void): void;
public fn beforeAll(fn: () => void): void;
public fn afterAll(fn: () => void): void;
```

---

**FIN DEL DOCUMENTO: Especificación de APIs Estándar**

Este documento define todas las APIs principales que los desarrolladores usarán en Vela.
