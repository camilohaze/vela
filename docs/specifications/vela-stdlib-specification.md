# Vela Standard Library Specification (Formal Contracts)

**Version:** 0.1.0-draft  
**Status:** Work in Progress  
**Last Updated:** 2025-11-30

---

## Table of Contents

1. [Collections](#1-collections)
2. [I/O and File System](#2-io-and-file-system)
3. [Networking](#3-networking)
4. [Concurrency Primitives](#4-concurrency-primitives)
5. [String Operations](#5-string-operations)

---

## 1. Collections

### 1.1 List<T>

**Description:** Dynamically-sized array with O(1) indexed access.

#### 1.1.1 push(item: T)

**Preconditions:**
- List is initialized

**Postconditions:**
- `list.length == old(list.length) + 1`
- `list[list.length - 1] == item`

**Invariants:**
- Capacity >= length

**Complexity:** O(1) amortized

**Thread Safety:** Not thread-safe (use from single actor)

```vela
let list = List<Int>();
list.push(42);  // length: 0 → 1
```

#### 1.1.2 pop() -> T?

**Preconditions:**
- List is initialized

**Postconditions:**
- If `old(list.length) > 0`: returns `Some(last_item)`, `list.length == old(list.length) - 1`
- If `old(list.length) == 0`: returns `null`

**Complexity:** O(1)

```vela
let list = List([1, 2, 3]);
let item = list.pop();  // Some(3), length: 3 → 2
```

#### 1.1.3 get(index: Int) -> T?

**Preconditions:**
- List is initialized

**Postconditions:**
- If `0 <= index < length`: returns `Some(list[index])`
- Otherwise: returns `null`

**Invariants:**
- Does not modify list

**Complexity:** O(1)

```vela
let list = List([10, 20, 30]);
list.get(1);  // Some(20)
list.get(5);  // null
```

#### 1.1.4 map<U>(fn: (T) -> U) -> List<U>

**Preconditions:**
- List is initialized
- `fn` is a valid function

**Postconditions:**
- Returns new list of same length
- `result[i] == fn(old(list[i]))` for all i

**Invariants:**
- Original list unchanged

**Complexity:** O(n)

**Thread Safety:** Safe if `fn` is pure

```vela
let nums = List([1, 2, 3]);
let doubled = nums.map(x => x * 2);  // [2, 4, 6]
```

---

### 1.2 Set<T>

**Description:** Unordered collection of unique elements.

#### 1.2.1 insert(item: T) -> Bool

**Preconditions:**
- Set is initialized
- `T` implements `Eq + Hash`

**Postconditions:**
- `set.contains(item) == true`
- Returns `true` if item was inserted, `false` if already present

**Complexity:** O(1) average, O(n) worst case

```vela
let set = Set<String>();
set.insert("hello");  // true
set.insert("hello");  // false (already present)
```

#### 1.2.2 contains(item: T) -> Bool

**Preconditions:**
- Set is initialized

**Postconditions:**
- Returns `true` if item in set, `false` otherwise
- Set unchanged

**Complexity:** O(1) average

**Thread Safety:** Safe for concurrent reads

---

### 1.3 Dict<K, V>

**Description:** Hash map for key-value storage.

#### 1.3.1 set(key: K, value: V)

**Preconditions:**
- Dict is initialized
- `K` implements `Eq + Hash`

**Postconditions:**
- `dict.get(key) == Some(value)`
- If key existed: old value replaced

**Complexity:** O(1) average

```vela
let dict = Dict<String, Int>();
dict.set("age", 25);
```

#### 1.3.2 get(key: K) -> V?

**Preconditions:**
- Dict is initialized

**Postconditions:**
- Returns `Some(value)` if key exists
- Returns `null` otherwise

**Invariants:**
- Dict unchanged

**Complexity:** O(1) average

**Thread Safety:** Safe for concurrent reads

---

## 2. I/O and File System

### 2.1 File.read(path: String) -> Result<String, Error>

**Preconditions:**
- `path` is valid UTF-8
- Process has read permissions

**Postconditions:**
- **Success**: Returns entire file contents as string
- **Failure**: Returns error (FileNotFound, PermissionDenied, etc.)

**Invariants:**
- File unchanged on disk
- No partial reads exposed

**Complexity:** O(file_size)

**Thread Safety:** Thread-safe (OS-level locking)

**Platform Behavior:**
- **Unix**: Uses `open(2)` + `read(2)`
- **Windows**: Uses `CreateFile` + `ReadFile`
- **Line endings**: Preserved as-is (no automatic conversion)

```vela
match File.read("/path/to/file.txt") {
    Ok(contents) => print(contents),
    Err(e) => print("Error: " + e.message),
}
```

### 2.2 File.write(path: String, data: String) -> Result<(), Error>

**Preconditions:**
- `path` is valid UTF-8
- Process has write permissions
- Parent directory exists

**Postconditions:**
- **Success**: File contains `data`, returns `Ok(())`
- **Failure**: File may not exist or be partially written, returns error

**Guarantees:**
- Atomic write (uses temp file + rename)
- Creates file if doesn't exist
- Truncates if file exists

**Complexity:** O(data.length)

**Thread Safety:** Thread-safe

---

## 3. Networking

### 3.1 HTTP.get(url: String) -> Future<Result<Response, Error>>

**Preconditions:**
- `url` is valid HTTP/HTTPS URL
- Network connectivity

**Postconditions:**
- **Success**: Returns HTTP response with status, headers, body
- **Failure**: Returns network error (Timeout, ConnectionRefused, etc.)

**Performance:**
- Timeout: 30 seconds default
- Connection pooling: Yes (HTTP/1.1 Keep-Alive)
- Retries: None (caller must implement)

**Thread Safety:** Thread-safe

**Platform Behavior:**
- Uses OS native TLS (OpenSSL on Linux, Secure Transport on macOS, Schannel on Windows)

```vela
let response = await HTTP.get("https://api.example.com/data");
match response {
    Ok(res) => {
        print("Status: " + res.status);
        print("Body: " + res.body);
    },
    Err(e) => print("Network error: " + e),
}
```

### 3.2 Server.listen(port: Int, handler: (Request) -> Response) -> Result<(), Error>

**Preconditions:**
- `1024 <= port <= 65535` (or root privileges for port < 1024)
- Port not in use

**Postconditions:**
- **Success**: Server listening on port, accepts connections
- **Failure**: Returns error (AddressInUse, PermissionDenied)

**Guarantees:**
- Non-blocking I/O (async)
- Handles multiple concurrent connections

**Complexity:** O(1) to start server, O(n) for n concurrent connections

**Thread Safety:** Thread-safe

---

## 4. Concurrency Primitives

### 4.1 Channel<T>

**Description:** Multi-producer, single-consumer queue.

#### 4.1.1 send(item: T)

**Preconditions:**
- Channel is open (not closed)

**Postconditions:**
- Item enqueued for receiver
- Returns immediately (non-blocking)

**Guarantees:**
- FIFO order preserved
- No message loss (unless receiver closed)

**Complexity:** O(1)

**Thread Safety:** Thread-safe (multiple senders)

```vela
let (sender, receiver) = channel<Int>();
sender.send(42);
```

#### 4.1.2 recv() -> T?

**Preconditions:**
- Channel exists

**Postconditions:**
- **If messages available**: Returns next message
- **If empty and senders exist**: Blocks until message arrives
- **If empty and all senders dropped**: Returns `null`

**Complexity:** O(1) if message available, otherwise blocks

**Thread Safety:** Single consumer only

---

### 4.2 Mutex<T>

**Description:** Mutual exclusion lock for shared mutable state.

#### 4.2.1 lock() -> MutexGuard<T>

**Preconditions:**
- Mutex initialized

**Postconditions:**
- Returns guard with exclusive access to inner data
- Blocks if already locked

**Guarantees:**
- No data races (exclusive access)
- Automatic unlock when guard drops

**Complexity:** O(1) if unlocked, otherwise blocks

**Thread Safety:** Thread-safe

```vela
let mutex = Mutex(0);
{
    let guard = mutex.lock();
    *guard += 1;  // Exclusive access
}  // Automatically unlocked here
```

---

## 5. String Operations

### 5.1 String.split(separator: String) -> List<String>

**Preconditions:**
- String and separator are valid UTF-8

**Postconditions:**
- Returns list of substrings
- Empty separator: splits into individual characters
- Separator not found: returns list with original string

**Invariants:**
- Original string unchanged
- `result.join(separator) == original` (if no trailing separator)

**Complexity:** O(n) where n = string length

**Thread Safety:** Safe (immutable strings)

```vela
"a,b,c".split(",");  // ["a", "b", "c"]
"hello".split("");   // ["h", "e", "l", "l", "o"]
```

### 5.2 String.trim() -> String

**Preconditions:**
- String is valid UTF-8

**Postconditions:**
- Returns new string with leading/trailing whitespace removed
- Whitespace: space, tab, newline, carriage return

**Invariants:**
- Original string unchanged

**Complexity:** O(n)

**Thread Safety:** Safe

```vela
"  hello  ".trim();  // "hello"
```

---

## Appendix A: Performance Guarantees Summary

| API | Time Complexity | Space Complexity |
|-----|----------------|------------------|
| List.push() | O(1) amortized | O(1) |
| List.get() | O(1) | O(1) |
| List.map() | O(n) | O(n) |
| Set.insert() | O(1) avg | O(1) |
| Dict.get() | O(1) avg | O(1) |
| File.read() | O(file_size) | O(file_size) |
| HTTP.get() | O(response_size) | O(response_size) |
| Channel.send() | O(1) | O(1) |
| String.split() | O(n) | O(n) |

---

## Appendix B: Thread Safety Summary

| API | Thread Safety |
|-----|--------------|
| List | Not thread-safe (use within single actor) |
| Set (read) | Safe for concurrent reads |
| Set (write) | Requires external synchronization |
| Dict (read) | Safe for concurrent reads |
| Dict (write) | Requires external synchronization |
| File I/O | Thread-safe (OS-level locking) |
| HTTP | Thread-safe |
| Channel.send() | Thread-safe (multi-producer) |
| Channel.recv() | Single consumer only |
| Mutex | Thread-safe |
| String | Safe (immutable) |

---

## Appendix C: Error Types

```vela
enum IOError {
    FileNotFound(path: String),
    PermissionDenied(path: String),
    AlreadyExists(path: String),
    NotADirectory(path: String),
    IsADirectory(path: String),
    DirectoryNotEmpty(path: String),
    ReadOnlyFilesystem,
    DiskFull,
    IOInterrupted,
    UnexpectedEof,
    Other(message: String),
}

enum NetworkError {
    Timeout,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddressInUse,
    AddressNotAvailable,
    InvalidUrl,
    DnsResolutionFailed,
    TlsHandshakeFailed,
    Other(message: String),
}
```

---

**References:**
- Rust stdlib: https://doc.rust-lang.org/std/
- ECMAScript built-ins: https://tc39.es/ecma262/
- POSIX API: https://pubs.opengroup.org/onlinepubs/9699919799/

---

*Document generated for Sprint 1 (TASK-000I)*  
*Historia: VELA-561 (US-00B)*  
*Last updated: 2025-11-30*
