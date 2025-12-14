# Vela Standard Library API Reference

## Table of Contents

1. [Core Types](#1-core-types)
   - [Option<T>](#optiont)
   - [Result<T, E>](#resultt-e)
   - [Primitives](#primitives)

2. [Collections](#2-collections)
   - [Array<T>](#arrayt)
   - [Map<K, V>](#mapk-v)
   - [Set<T>](#sett)
   - [Iterator Protocol](#iterator-protocol)

3. [Strings](#3-strings)
   - [String](#string)
   - [StringBuilder](#stringbuilder)

4. [Math](#4-math)
   - [Basic Math](#basic-math)
   - [Trigonometry](#trigonometry)
   - [Random](#random)

5. [IO](#5-io)
   - [File System](#file-system)
   - [Paths](#paths)
   - [Streams](#streams)

6. [Time](#6-time)
   - [DateTime](#datetime)
   - [Duration](#duration)
   - [Time Zones](#time-zones)

7. [JSON](#7-json)
   - [Serialization](#serialization)
   - [Deserialization](#deserialization)
   - [Streaming](#streaming)

8. [HTTP](#8-http)
   - [Client](#client)
   - [Server](#server)
   - [Middleware](#middleware)

9. [Reactive](#9-reactive)
   - [Signals](#signals)
   - [Computed](#computed)
   - [Effects](#effects)

10. [UI Framework](#10-ui-framework)
    - [Widgets](#widgets)
    - [Layout](#layout)
    - [Events](#events)

---

## 1. Core Types

### Option<T>

Represents optional values. Eliminates null pointer exceptions.

#### Constructors
```vela
Some(value: T) -> Option<T>
None -> Option<T>
```

#### Methods
```vela
// Check if has value
isSome() -> Bool
isNone() -> Bool

// Get value (unsafe)
unwrap() -> T  // Panics if None

// Get value with default
unwrapOr(default: T) -> T

// Transform value
map<U>(f: (T) -> U) -> Option<U>
flatMap<U>(f: (T) -> Option<U>) -> Option<U>

// Filter
filter(predicate: (T) -> Bool) -> Option<T>

// Get or compute
getOrElse(f: () -> T) -> T
```

#### Examples
```vela
// Safe optional access
user = findUser(id)
name = user.map(u => u.name).unwrapOr("Unknown")

// Chaining
email = user
  .filter(u => u.verified)
  .flatMap(u => u.email)
```

### Result<T, E>

Represents computation results. Type-safe error handling.

#### Constructors
```vela
Ok(value: T) -> Result<T, E>
Err(error: E) -> Result<T, E>
```

#### Methods
```vela
// Check result type
isOk() -> Bool
isErr() -> Bool

// Get values (unsafe)
unwrap() -> T      // Panics if Err
unwrapErr() -> E   // Panics if Ok

// Get with defaults
unwrapOr(default: T) -> T
unwrapOrElse(f: (E) -> T) -> T

// Transform
map<U>(f: (T) -> U) -> Result<U, E>
mapErr<F>(f: (E) -> F) -> Result<T, F>
flatMap<U>(f: (T) -> Result<U, E>) -> Result<U, E>

// Handle errors
recover(f: (E) -> T) -> T
orElse(f: (E) -> Result<T, E>) -> Result<T, E>
```

#### Examples
```vela
// Safe division
fn divide(a: Float, b: Float) -> Result<Float, String> {
  if b == 0.0 {
    return Err("Division by zero")
  }
  return Ok(a / b)
}

// Usage
result = divide(10, 2)
final = result.map(x => x * 2).unwrapOr(0)
```

### Primitives

#### Number
```vela
// Arithmetic
+ - * / % **

// Comparison
== != < <= > >=

// Conversion
toString() -> String
toFloat() -> Float
```

#### Float
```vela
// Arithmetic
+ - * / % **

// Comparison
== != < <= > >=

// Math functions
abs() -> Float
ceil() -> Float
floor() -> Float
round() -> Float

// Conversion
toString() -> String
toInt() -> Number
```

#### String
```vela
// Properties
length: Number
isEmpty: Bool

// Access
charAt(index: Number) -> String
substring(start: Number, end: Number) -> String
slice(start: Number, end: Number) -> String

// Search
indexOf(search: String) -> Option<Number>
lastIndexOf(search: String) -> Option<Number>
contains(search: String) -> Bool
startsWith(prefix: String) -> Bool
endsWith(suffix: String) -> Bool

// Modification
toUpperCase() -> String
toLowerCase() -> String
trim() -> String
trimStart() -> String
trimEnd() -> String
replace(old: String, new: String) -> String
replaceAll(old: String, new: String) -> String
split(separator: String) -> Array<String>

// Conversion
toNumber() -> Option<Number>
toFloat() -> Option<Float>
```

#### Bool
```vela
// Logical operations
! (not)
&& (and)
|| (or)

// Conversion
toString() -> String
```

---

## 2. Collections

### Array<T>

Immutable array with functional operations.

#### Constructors
```vela
[] -> Array<T>
[value1, value2, ...] -> Array<T>
Array.of(values: T...) -> Array<T>
Array.range(start: Number, end: Number) -> Array<Number>
```

#### Properties
```vela
length: Number
isEmpty: Bool
```

#### Functional Operations
```vela
// Transformation
map<U>(f: (T) -> U) -> Array<U>
flatMap<U>(f: (T) -> Array<U>) -> Array<U>
flatten() -> Array<T>  // for Array<Array<T>>

// Filtering
filter(predicate: (T) -> Bool) -> Array<T>
take(n: Number) -> Array<T>
takeWhile(predicate: (T) -> Bool) -> Array<T>
drop(n: Number) -> Array<T>
dropWhile(predicate: (T) -> Bool) -> Array<T>

// Reduction
reduce<U>(initial: U, f: (U, T) -> U) -> U
foldLeft<U>(initial: U, f: (U, T) -> U) -> U
foldRight<U>(initial: U, f: (T, U) -> U) -> U

// Finding
find(predicate: (T) -> Bool) -> Option<T>
findIndex(predicate: (T) -> Bool) -> Option<Number>
indexOf(element: T) -> Option<Number>
contains(element: T) -> Bool

// Testing
every(predicate: (T) -> Bool) -> Bool
some(predicate: (T) -> Bool) -> Bool

// Grouping
groupBy<K>(f: (T) -> K) -> Map<K, Array<T>>
partition(predicate: (T) -> Bool) -> (Array<T>, Array<T>)

// Sorting
sortBy(comparator: (T, T) -> Number) -> Array<T>
sort() -> Array<T>  // for Comparable types

// Combination
zip<U>(other: Array<U>) -> Array<(T, U)>
concat(other: Array<T>) -> Array<T>

// Side effects
forEach(action: (T) -> void) -> void
```

#### Access Operations
```vela
// Indexing
get(index: Number) -> Option<T>
first() -> Option<T>
last() -> Option<T>

// Slicing
slice(start: Number, end: Number) -> Array<T>
```

#### Examples
```vela
// Functional pipeline
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

result = numbers
  .filter(x => x % 2 == 0)     // [2, 4, 6, 8, 10]
  .map(x => x * x)             // [4, 16, 36, 64, 100]
  .take(3)                     // [4, 16, 36]
  .reduce(0, (sum, x) => sum + x)  // 56
```

### Map<K, V>

Immutable hash map.

#### Constructors
```vela
{} -> Map<K, V>
{key1: value1, key2: value2} -> Map<K, V>
Map.empty() -> Map<K, V>
```

#### Properties
```vela
size: Number
isEmpty: Bool
keys: Array<K>
values: Array<V>
entries: Array<(K, V)>
```

#### Operations
```vela
// Access
get(key: K) -> Option<V>
getOrElse(key: K, default: V) -> V

// Modification (returns new Map)
set(key: K, value: V) -> Map<K, V>
remove(key: K) -> Map<K, V>
update(key: K, f: (V) -> V) -> Map<K, V>

// Queries
containsKey(key: K) -> Bool
containsValue(value: V) -> Bool

// Transformation
mapValues<U>(f: (V) -> U) -> Map<K, U>
filter(predicate: ((K, V)) -> Bool) -> Map<K, V>

// Iteration
forEach(action: ((K, V)) -> void) -> void
```

#### Examples
```vela
// User database
users = {
  "alice": { name: "Alice", age: 30 },
  "bob": { name: "Bob", age: 25 }
}

// Update user
updatedUsers = users.set("alice", {
  name: "Alice",
  age: 31
})

// Find adult users
adults = users.filter((_, user) => user.age >= 18)
```

### Set<T>

Immutable set.

#### Constructors
```vela
Set.empty() -> Set<T>
Set.of(values: T...) -> Set<T>
```

#### Properties
```vela
size: Number
isEmpty: Bool
```

#### Operations
```vela
// Modification
add(element: T) -> Set<T>
remove(element: T) -> Set<T>

// Set operations
union(other: Set<T>) -> Set<T>
intersection(other: Set<T>) -> Set<T>
difference(other: Set<T>) -> Set<T>

// Queries
contains(element: T) -> Bool
isSubsetOf(other: Set<T>) -> Bool
isSupersetOf(other: Set<T>) -> Bool

// Transformation
map<U>(f: (T) -> U) -> Set<U>
filter(predicate: (T) -> Bool) -> Set<T>

// Iteration
forEach(action: (T) -> void) -> void
```

### Iterator Protocol

Lazy evaluation for collections.

```vela
interface Iterator<T> {
  next() -> Option<T>
  hasNext() -> Bool
}

interface Iterable<T> {
  iterator() -> Iterator<T>
}
```

#### Examples
```vela
// Custom iterator
fn fibonacci() -> Iterator<Number> {
  state a: Number = 0
  state b: Number = 1

  return {
    next: () => {
      current = a
      temp = a + b
      a = b
      b = temp
      return Some(current)
    },
    hasNext: () => true  // infinite
  }
}

// Usage
fib = fibonacci()
first10 = fib.take(10).toArray()  // [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

---

## 3. Strings

### String

UTF-8 string operations.

#### Constructors
```vela
"" -> String
"hello" -> String
String.fromCharCodes(codes: Array<Number>) -> String
```

#### Properties
```vela
length: Number
isEmpty: Bool
```

#### Character Operations
```vela
charAt(index: Number) -> String
charCodeAt(index: Number) -> Number
codePoints() -> Array<Number>
```

#### Substring Operations
```vela
substring(start: Number, end: Number) -> String
slice(start: Number, end: Number) -> String
substr(start: Number, length: Number) -> String
```

#### Search and Replace
```vela
indexOf(search: String, start: Number = 0) -> Option<Number>
lastIndexOf(search: String, start: Number = length) -> Option<Number>
includes(search: String) -> Bool
startsWith(prefix: String) -> Bool
endsWith(suffix: String) -> Bool

replace(old: String, new: String) -> String
replaceAll(old: String, new: String) -> String
replaceFirst(pattern: String, replacement: String) -> String
```

#### Case Conversion
```vela
toUpperCase() -> String
toLowerCase() -> String
toTitleCase() -> String
```

#### Trimming
```vela
trim() -> String
trimStart() -> String
trimEnd() -> String
trimChars(chars: String) -> String
```

#### Splitting and Joining
```vela
split(separator: String) -> Array<String>
splitLimit(separator: String, limit: Number) -> Array<String>
lines() -> Array<String>
words() -> Array<String>

join(separator: String) -> String  // for Array<String>
```

#### Formatting
```vela
padStart(targetLength: Number, padString: String = " ") -> String
padEnd(targetLength: Number, padString: String = " ") -> String
repeat(count: Number) -> String
```

#### Examples
```vela
// String processing pipeline
text = "  Hello,   World!  "

clean = text
  .trim()                    // "Hello,   World!"
  .replaceAll("  ", " ")     // "Hello, World!"
  .toLowerCase()             // "hello, world!"

words = clean.words()        // ["hello,", "world!"]
```

### StringBuilder

Efficient string concatenation.

#### Constructor
```vela
StringBuilder.new() -> StringBuilder
StringBuilder.withCapacity(capacity: Number) -> StringBuilder
```

#### Methods
```vela
append(value: String) -> StringBuilder
appendChar(char: String) -> StringBuilder
appendLine(value: String = "") -> StringBuilder
insert(offset: Number, value: String) -> StringBuilder
delete(start: Number, end: Number) -> StringBuilder
clear() -> StringBuilder
toString() -> String
length() -> Number
capacity() -> Number
```

#### Examples
```vela
// Efficient concatenation
builder = StringBuilder.new()

(1..1000).forEach(i => {
  builder.append("Line ").append(i.toString()).appendLine()
})

result = builder.toString()
```

---

## 4. Math

### Basic Math

#### Constants
```vela
Math.PI: Float       // 3.141592653589793
Math.E: Float        // 2.718281828459045
Math.LN2: Float      // 0.6931471805599453
Math.LN10: Float     // 2.302585092994046
Math.LOG2E: Float    // 1.4426950408889634
Math.LOG10E: Float   // 0.4342944819032518
Math.SQRT2: Float    // 1.4142135623730951
```

#### Functions
```vela
Math.abs(x: Float) -> Float
Math.sign(x: Float) -> Float  // -1, 0, or 1
Math.ceil(x: Float) -> Float
Math.floor(x: Float) -> Float
Math.round(x: Float) -> Float
Math.trunc(x: Float) -> Float

Math.max(a: Float, b: Float) -> Float
Math.min(a: Float, b: Float) -> Float
Math.clamp(value: Float, min: Float, max: Float) -> Float

Math.sqrt(x: Float) -> Float
Math.cbrt(x: Float) -> Float
Math.pow(base: Float, exponent: Float) -> Float
Math.exp(x: Float) -> Float
Math.expm1(x: Float) -> Float
Math.log(x: Float) -> Float
Math.log2(x: Float) -> Float
Math.log10(x: Float) -> Float
Math.log1p(x: Float) -> Float
```

### Trigonometry

```vela
Math.sin(angle: Float) -> Float    // radians
Math.cos(angle: Float) -> Float
Math.tan(angle: Float) -> Float

Math.asin(x: Float) -> Float
Math.acos(x: Float) -> Float
Math.atan(x: Float) -> Float
Math.atan2(y: Float, x: Float) -> Float

Math.sinh(x: Float) -> Float
Math.cosh(x: Float) -> Float
Math.tanh(x: Float) -> Float

Math.toRadians(degrees: Float) -> Float
Math.toDegrees(radians: Float) -> Float
```

### Random

```vela
interface Random {
  nextInt() -> Number
  nextIntBounded(max: Number) -> Number
  nextIntRange(min: Number, max: Number) -> Number
  nextFloat() -> Float
  nextFloatRange(min: Float, max: Float) -> Float
  nextBool() -> Bool
  nextBytes(length: Number) -> Array<Number>
}

Math.random() -> Float  // 0.0 to 1.0
Random.new() -> Random
Random.withSeed(seed: Number) -> Random
```

#### Examples
```vela
// Random number generation
rng = Random.new()

// Dice roll
diceRoll = rng.nextIntRange(1, 7)

// Random float
randomValue = rng.nextFloatRange(0.0, 100.0)

// Shuffle array
shuffled = array.shuffle(rng)
```

---

## 5. IO

### File System

#### File Operations
```vela
interface File {
  readAsString() -> Result<String, IOError>
  readAsBytes() -> Result<Array<Number>, IOError>
  writeString(content: String) -> Result<void, IOError>
  writeBytes(bytes: Array<Number>) -> Result<void, IOError>
  appendString(content: String) -> Result<void, IOError>
  appendBytes(bytes: Array<Number>) -> Result<void, IOError>
  length() -> Result<Number, IOError>
  exists() -> Bool
  delete() -> Result<void, IOError>
  copyTo(destination: Path) -> Result<void, IOError>
  moveTo(destination: Path) -> Result<void, IOError>
}

File.open(path: Path) -> Result<File, IOError>
File.create(path: Path) -> Result<File, IOError>
File.openRead(path: Path) -> Result<File, IOError>
File.openWrite(path: Path) -> Result<File, IOError>
```

#### Directory Operations
```vela
interface Directory {
  list() -> Result<Array<Path>, IOError>
  create() -> Result<void, IOError>
  exists() -> Bool
  delete() -> Result<void, IOError>
  deleteRecursive() -> Result<void, IOError>
}

Directory.open(path: Path) -> Result<Directory, IOError>
Directory.create(path: Path) -> Result<void, IOError>
```

### Paths

```vela
interface Path {
  toString() -> String
  isAbsolute() -> Bool
  isRelative() -> Bool
  parent() -> Option<Path>
  fileName() -> Option<String>
  extension() -> Option<String>
  join(other: Path) -> Path
  resolve(other: Path) -> Path
  relativize(other: Path) -> Path
  normalize() -> Path
}

Path.of(path: String) -> Path
Path.currentDirectory() -> Path
Path.homeDirectory() -> Path
Path.temporaryDirectory() -> Path
```

### Streams

#### Readable Stream
```vela
interface ReadableStream<T> {
  read() -> Result<Option<T>, IOError>
  readAll() -> Result<Array<T>, IOError>
  close() -> Result<void, IOError>
}
```

#### Writable Stream
```vela
interface WritableStream<T> {
  write(data: T) -> Result<void, IOError>
  writeAll(data: Array<T>) -> Result<void, IOError>
  flush() -> Result<void, IOError>
  close() -> Result<void, IOError>
}
```

#### Examples
```vela
// Read entire file
content = File.open("data.txt")
  .flatMap(file => file.readAsString())
  .unwrapOr("")

// Write to file
result = File.create("output.txt")
  .flatMap(file => file.writeString("Hello, World!"))

// List directory
entries = Directory.open(".")
  .flatMap(dir => dir.list())
  .unwrapOr([])
```

---

## 6. Time

### DateTime

```vela
interface DateTime {
  year: Number
  month: Number    // 1-12
  day: Number      // 1-31
  hour: Number     // 0-23
  minute: Number   // 0-59
  second: Number   // 0-59
  nanosecond: Number
  timezone: TimeZone

  toString() -> String
  format(pattern: String) -> String
  toEpochMillis() -> Number
  toEpochSeconds() -> Number
}

DateTime.now() -> DateTime
DateTime.of(year: Number, month: Number, day: Number) -> DateTime
DateTime.of(year: Number, month: Number, day: Number, hour: Number, minute: Number, second: Number) -> DateTime
DateTime.fromEpochMillis(millis: Number) -> DateTime
DateTime.fromEpochSeconds(seconds: Number) -> DateTime
DateTime.parse(isoString: String) -> Result<DateTime, ParseError>
```

### Duration

```vela
interface Duration {
  seconds: Number
  nanoseconds: Number

  toString() -> String
  toMillis() -> Number
  toSeconds() -> Float
  toMinutes() -> Float
  toHours() -> Float
  toDays() -> Float
}

Duration.ofSeconds(seconds: Number) -> Duration
Duration.ofMillis(millis: Number) -> Duration
Duration.ofMinutes(minutes: Number) -> Duration
Duration.ofHours(hours: Number) -> Duration
Duration.ofDays(days: Number) -> Duration
Duration.between(start: DateTime, end: DateTime) -> Duration
```

### Time Zones

```vela
interface TimeZone {
  id: String
  offset: Duration

  toString() -> String
}

TimeZone.utc() -> TimeZone
TimeZone.system() -> TimeZone
TimeZone.of(id: String) -> Result<TimeZone, TimeZoneError>
TimeZone.ofOffset(hours: Number) -> TimeZone
```

#### Examples
```vela
// Current time
now = DateTime.now()

// Parse date
birthday = DateTime.parse("1990-05-15T10:30:00Z").unwrap()

// Calculate age
age = Duration.between(birthday, now).toDays() / 365.25

// Format time
formatted = now.format("yyyy-MM-dd HH:mm:ss")
```

---

## 7. JSON

### Serialization

```vela
interface JsonSerializable {
  toJson() -> JsonValue
}

JsonValue.toJson(value: any) -> JsonValue
JsonValue.toString() -> String
JsonValue.toPrettyString() -> String
```

### Deserialization

```vela
JsonValue.parse(jsonString: String) -> Result<JsonValue, JsonError>
JsonValue.fromString(jsonString: String) -> Result<any, JsonError>
JsonValue.fromFile(path: Path) -> Result<any, JsonError>
```

### JsonValue Types

```vela
enum JsonValue {
  Null,
  Bool(value: Bool),
  Number(value: Float),
  String(value: String),
  Array(values: Array<JsonValue>),
  Object(fields: Map<String, JsonValue>)
}
```

#### Examples
```vela
// Serialize object
user = {
  name: "Alice",
  age: 30,
  active: true,
  tags: ["developer", "vela"]
}

jsonString = JsonValue.toPrettyString(user)

// Deserialize
parsed = JsonValue.fromString(jsonString)
name = parsed.unwrap()["name"].unwrap()  // "Alice"
```

### Streaming

```vela
interface JsonStreamReader {
  next() -> Result<Option<JsonValue>, JsonError>
  close() -> Result<void, IOError>
}

interface JsonStreamWriter {
  write(value: JsonValue) -> Result<void, IOError>
  close() -> Result<void, IOError>
}

JsonStreamReader.openFile(path: Path) -> Result<JsonStreamReader, IOError>
JsonStreamWriter.createFile(path: Path) -> Result<JsonStreamWriter, IOError>
```

---

## 8. HTTP

### Client

```vela
interface HttpClient {
  get(url: String) -> Result<HttpResponse, HttpError>
  post(url: String, body: String) -> Result<HttpResponse, HttpError>
  put(url: String, body: String) -> Result<HttpResponse, HttpError>
  delete(url: String) -> Result<HttpResponse, HttpError>
  request(method: String, url: String, options: HttpRequestOptions) -> Result<HttpResponse, HttpError>
}

interface HttpResponse {
  status: Number
  statusText: String
  headers: Map<String, String>
  body: String
  bodyAsBytes: Array<Number>
  json<T>() -> Result<T, JsonError>
}

interface HttpRequestOptions {
  headers: Map<String, String>
  body: String
  timeout: Duration
  followRedirects: Bool
}

HttpClient.new() -> HttpClient
HttpClient.withTimeout(timeout: Duration) -> HttpClient
```

#### Examples
```vela
// Simple GET request
client = HttpClient.new()
response = client.get("https://api.github.com/users/velalang")

if response.isOk() {
  user = response.json()
  print("User: ${user.login}")
}

// POST with JSON
userData = { name: "Alice", email: "alice@example.com" }
options = {
  headers: { "Content-Type": "application/json" },
  body: JsonValue.toString(userData)
}

response = client.post("https://api.example.com/users", options)
```

### Server

```vela
interface HttpServer {
  listen(port: Number) -> Result<void, HttpError>
  stop() -> Result<void, HttpError>
  use(middleware: HttpMiddleware) -> HttpServer
  get(path: String, handler: HttpHandler) -> HttpServer
  post(path: String, handler: HttpHandler) -> HttpServer
  put(path: String, handler: HttpHandler) -> HttpServer
  delete(path: String, handler: HttpHandler) -> HttpServer
}

interface HttpRequest {
  method: String
  url: String
  path: String
  query: Map<String, String>
  headers: Map<String, String>
  body: String
  params: Map<String, String>
  json<T>() -> Result<T, JsonError>
}

interface HttpResponse {
  status(code: Number) -> HttpResponse
  header(name: String, value: String) -> HttpResponse
  json(data: any) -> HttpResponse
  text(content: String) -> HttpResponse
  html(content: String) -> HttpResponse
  send() -> void
}

type HttpHandler = (request: HttpRequest, response: HttpResponse) -> void
type HttpMiddleware = (request: HttpRequest, response: HttpResponse, next: () -> void) -> void

HttpServer.new() -> HttpServer
```

#### Examples
```vela
// Simple HTTP server
server = HttpServer.new()

// Middleware
server.use((req, res, next) => {
  print("${req.method} ${req.path}")
  next()
})

// Routes
server.get("/api/users", (req, res) => {
  users = [{ id: 1, name: "Alice" }, { id: 2, name: "Bob" }]
  res.json(users)
})

server.post("/api/users", (req, res) => {
  user = req.json()
  // Save user...
  res.status(201).json({ id: 3, ...user })
})

// Start server
server.listen(3000)
```

### Middleware

```vela
// CORS middleware
corsMiddleware = (req, res, next) => {
  res.header("Access-Control-Allow-Origin", "*")
  res.header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
  res.header("Access-Control-Allow-Headers", "Content-Type")

  if req.method == "OPTIONS" {
    res.status(200).send()
  } else {
    next()
  }
}

// Authentication middleware
authMiddleware = (req, res, next) => {
  token = req.headers.get("Authorization")

  if token.isNone() {
    res.status(401).json({ error: "Unauthorized" })
  } else {
    // Verify token...
    next()
  }
}

server.use(corsMiddleware)
server.use(authMiddleware)
```

---

## 9. Reactive

### Signals

```vela
interface Signal<T> {
  get() -> T
  set(value: T) -> void
  update(f: (T) -> T) -> void
  subscribe(listener: (T) -> void) -> Subscription
}

Signal.new<T>(initial: T) -> Signal<T>
Signal.computed<T>(computation: () -> T) -> Signal<T>
```

### Computed

```vela
interface Computed<T> {
  get() -> T
}

Computed.new<T>(computation: () -> T) -> Computed<T>
```

### Effects

```vela
interface Effect {
  stop() -> void
}

Effect.run(action: () -> void) -> Effect
Effect.run(dependencies: Array<Signal<any>>, action: () -> void) -> Effect
```

#### Examples
```vela
// Basic signals
count = Signal.new(0)
name = Signal.new("Alice")

// Computed values
fullName = Signal.computed(() => {
  return "${name.get()} (count: ${count.get()})"
})

// Effects
effect = Effect.run(() => {
  print("Full name changed: ${fullName.get()}")
})

// Changes trigger effects automatically
count.set(5)  // Effect runs: "Full name changed: Alice (count: 5)"
name.set("Bob")  // Effect runs: "Full name changed: Bob (count: 5)"
```

---

## 10. UI Framework

### Widgets

#### Base Widget Interface
```vela
interface Widget {
  build() -> RenderObject
}

interface StatefulWidget extends Widget {
  createState() -> State
}

interface StatelessWidget extends Widget {
  build() -> Widget
}
```

#### Basic Widgets
```vela
// Text
Text(text: String, style: TextStyle = {}) -> Widget

// Containers
Container(
  child: Widget,
  padding: EdgeInsets = {},
  margin: EdgeInsets = {},
  color: Color = null,
  width: Float = null,
  height: Float = null
) -> Widget

// Buttons
ElevatedButton(
  text: String,
  onPressed: () -> void,
  style: ButtonStyle = {}
) -> Widget

TextButton(
  text: String,
  onPressed: () -> void,
  style: ButtonStyle = {}
) -> Widget
```

#### Layout Widgets
```vela
// Column (vertical layout)
Column(
  children: Array<Widget>,
  mainAxisAlignment: MainAxisAlignment = .start,
  crossAxisAlignment: CrossAxisAlignment = .start,
  spacing: Float = 0
) -> Widget

// Row (horizontal layout)
Row(
  children: Array<Widget>,
  mainAxisAlignment: MainAxisAlignment = .start,
  crossAxisAlignment: CrossAxisAlignment = .start,
  spacing: Float = 0
) -> Widget

// Stack (overlay)
Stack(
  children: Array<Widget>,
  alignment: Alignment = .topLeft
) -> Widget

// ListView (scrollable list)
ListView(
  children: Array<Widget>,
  scrollDirection: Axis = .vertical
) -> Widget
```

#### Input Widgets
```vela
// Text input
TextField(
  placeholder: String = "",
  initialValue: String = "",
  onChanged: (String) -> void = null,
  onSubmitted: (String) -> void = null
) -> Widget

// Checkbox
Checkbox(
  value: Bool = false,
  onChanged: (Bool) -> void = null
) -> Widget

// Radio buttons
Radio<T>(
  value: T,
  groupValue: T,
  onChanged: (T) -> void = null
) -> Widget
```

### Layout

#### EdgeInsets
```vela
EdgeInsets.all(value: Float) -> EdgeInsets
EdgeInsets.symmetric(vertical: Float = 0, horizontal: Float = 0) -> EdgeInsets
EdgeInsets.only(
  top: Float = 0,
  bottom: Float = 0,
  left: Float = 0,
  right: Float = 0
) -> EdgeInsets
```

#### Alignment
```vela
enum MainAxisAlignment {
  start, center, end, spaceBetween, spaceAround, spaceEvenly
}

enum CrossAxisAlignment {
  start, center, end, stretch
}

enum Alignment {
  topLeft, topCenter, topRight,
  centerLeft, center, centerRight,
  bottomLeft, bottomCenter, bottomRight
}
```

### Events

#### Event Handlers
```vela
type OnPressed = () -> void
type OnChanged<T> = (T) -> void
type OnSubmitted = (String) -> void
type OnTap = () -> void
type OnHover = (Bool) -> void
```

#### Gesture Recognizers
```vela
GestureDetector(
  child: Widget,
  onTap: OnTap = null,
  onDoubleTap: OnTap = null,
  onLongPress: OnTap = null,
  onPanStart: (DragStartDetails) -> void = null,
  onPanUpdate: (DragUpdateDetails) -> void = null,
  onPanEnd: (DragEndDetails) -> void = null
) -> Widget
```

#### Examples
```vela
// Counter app
state count: Number = 0

app = Container(
  padding: EdgeInsets.all(20),
  child: Column(
    children: [
      Text("Counter: ${count}", style: { fontSize: 24 }),
      SizedBox(height: 20),
      Row(
        mainAxisAlignment: .center,
        children: [
          ElevatedButton(
            text: "Increment",
            onPressed: () => { count = count + 1 }
          ),
          SizedBox(width: 10),
          ElevatedButton(
            text: "Decrement",
            onPressed: () => { count = count - 1 }
          )
        ]
      )
    ]
  )
)

runApp(app)
```

---

This API reference covers the complete Vela standard library. All APIs follow functional programming principles with immutable data structures and type-safe error handling.