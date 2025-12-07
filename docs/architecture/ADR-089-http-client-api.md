# ADR-089: Diseño de la API HttpClient

## Estado
✅ Aceptado

## Fecha
2025-12-07

## Contexto
Necesitamos implementar un cliente HTTP para Vela que permita hacer requests HTTP de manera síncrona y asíncrona. La API debe ser simple pero poderosa, inspirada en las mejores prácticas de lenguajes modernos como TypeScript (fetch API), Java (HttpClient), y frameworks como NestJS y Spring Boot.

El cliente debe manejar:
- Métodos HTTP básicos (GET, POST, PUT, DELETE, PATCH)
- Headers personalizados
- Query parameters
- Body en diferentes formatos (JSON, text, form-data)
- Timeouts configurables
- Manejo de errores consistente
- Response parsing automático

## Decisión
Implementaremos una API HttpClient inspirada en fetch() pero con mejoras para casos de uso empresarial:

### API Principal
```vela
// Cliente básico
let client = HttpClient::new();

// GET request
let response = client.get("https://api.example.com/users").await?;
let users = response.json::<Vec<User>>().await?;

// POST con JSON
let newUser = User { name: "John", email: "john@example.com" };
let response = client.post("https://api.example.com/users")
    .json(&newUser)
    .send()
    .await?;

// Con headers y timeout
let response = client.get("https://api.example.com/data")
    .header("Authorization", "Bearer token")
    .timeout(Duration::from_secs(30))
    .send()
    .await?;
```

### Características Clave
1. **Builder Pattern**: Métodos encadenables para configuración
2. **Async/Await**: Soporte nativo para operaciones asíncronas
3. **Type Safety**: Generic methods para parsing JSON
4. **Error Handling**: Result<T, HttpError> consistente
5. **Middleware Support**: Extensible para interceptores

### Estructuras
- `HttpClient`: Cliente principal con configuración global
- `HttpRequest`: Builder para requests individuales
- `HttpResponse`: Wrapper para responses con métodos de parsing
- `HttpError`: Tipos de error específicos (timeout, network, status, etc.)

## Consecuencias

### Positivas
- ✅ API intuitiva similar a fetch() pero más poderosa
- ✅ Type safety con generics para JSON parsing
- ✅ Configurable y extensible
- ✅ Manejo robusto de errores
- ✅ Inspirado en las mejores prácticas de la industria

### Negativas
- ❌ Complejidad inicial mayor que una API minimalista
- ❌ Dependencia de crates async (tokio, reqwest)
- ❌ Curva de aprendizaje para usuarios nuevos

## Alternativas Consideradas

### 1. API Minimalista (Rechazada)
```vela
// Muy simple pero limitado
let response = http_get("https://api.example.com");
```
**Rechazada porque:** No soporta POST, headers, timeouts, error handling avanzado.

### 2. API Similar a Java HttpClient (Rechazada)
```vela
let client = HttpClient.newHttpClient();
let request = HttpRequest.newBuilder()
    .uri(URI.create("https://api.example.com"))
    .build();
let response = client.send(request, HttpResponse.BodyHandlers.ofString());
```
**Rechazada porque:** Muy verbose, no idiomático en lenguajes modernos.

### 3. API Similar a Axios (JavaScript) (Rechazada)
```vela
axios.get('/users', {
    headers: { 'Authorization': 'token' },
    timeout: 5000
});
```
**Rechazada porque:** Menos type-safe que fetch(), pero considerada. Elegimos fetch() por ser estándar web.

## Referencias
- **Jira:** [TASK-089](https://velalang.atlassian.net/browse/TASK-089)
- **Historia:** [VELA-591](https://velalang.atlassian.net/browse/VELA-591)
- **Inspiración:**
  - TypeScript: `fetch()` API
  - Java: `java.net.http.HttpClient`
  - NestJS: `HttpService`
  - Spring Boot: `RestTemplate` / `WebClient`
  - Rust: `reqwest` crate
- **Documentación:** Ver código en `src/http_client.rs`</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-089-http-client-api.md