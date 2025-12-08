# TASK-113O: Implementar log transports

## 📋 Información General
- **Historia:** VELA-597 (US-24C)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-08

## 🎯 Objetivo
Implementar sistema extensible de transports para dirigir logs a diferentes destinos: consola, archivos, HTTP endpoints, y otros sistemas.

## 🔨 Implementación

### LogTransport Trait

#### Interfaz Base
```rust
#[async_trait]
pub trait LogTransport: Send + Sync {
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn name(&self) -> &str;
}
```

#### Características del Trait
- **Async por defecto**: Todos los transports son async
- **Thread-safe**: Send + Sync bounds
- **Error handling**: Result return type
- **Identificación**: Método name() para debugging

### Transports Implementados

#### 1. ConsoleTransport
```rust
pub struct ConsoleTransport {
    colored: bool,
}
```

**Características:**
- **Output coloreado**: Niveles diferentes con colores ANSI
- **Configurable**: Opción para deshabilitar colores
- **Thread-safe**: Usa stdout/stderr apropiados
- **Búfer inmediato**: Flush automático

**Colores por nivel:**
- DEBUG: Azul
- INFO: Verde
- WARN: Amarillo
- ERROR: Rojo
- FATAL: Rojo brillante con negrita

#### 2. FileTransport
```rust
pub struct FileTransport {
    file_path: PathBuf,
    append: bool,
}
```

**Características:**
- **Append mode**: Agrega a archivos existentes por defecto
- **Create directories**: Crea directorios padre si no existen
- **Atomic writes**: Usa tokio::fs para I/O async
- **Error recovery**: Logging de errores internos

#### 3. HttpTransport
```rust
pub struct HttpTransport {
    url: String,
    client: reqwest::Client,
}
```

**Características:**
- **HTTP POST**: Envía logs como JSON via POST
- **Configurable endpoint**: URL personalizable
- **Async HTTP**: Usa reqwest con tokio
- **Error handling**: Reintentos y logging de fallos
- **Timeout**: Configurable timeout por defecto

### Configuración de Transports

#### LogConfig con Transports
```rust
let config = LogConfig::new()
    .with_transport(Box::new(ConsoleTransport::new(true)))
    .with_transport(Box::new(FileTransport::new("app.log")))
    .with_transport(Box::new(HttpTransport::new("http://localhost:8080/logs")));
```

#### Múltiples Transports
- **Paralelo**: Todos los transports reciben todos los logs
- **Independiente**: Fallo en un transport no afecta otros
- **Configurable**: Transports por instancia de Logger

### Extensibilidad

#### Creando Custom Transports
```rust
pub struct DatabaseTransport {
    pool: sqlx::PgPool,
}

#[async_trait]
impl LogTransport for DatabaseTransport {
    async fn write(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Insertar en base de datos
        sqlx::query("INSERT INTO logs (timestamp, level, message, metadata) VALUES (?, ?, ?, ?)")
            .bind(record.timestamp)
            .bind(record.level.as_str())
            .bind(&record.message)
            .bind(record.to_json())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "database"
    }
}
```

## ✅ Criterios de Aceptación
- [x] LogTransport trait definido y funcional
- [x] ConsoleTransport con colores implementado
- [x] FileTransport con append mode implementado
- [x] HttpTransport básico implementado
- [x] Sistema extensible para custom transports
- [x] Tests de transports pasando
- [x] Error handling robusto en todos los transports

## 🔗 Referencias
- **Jira:** [TASK-113O](https://velalang.atlassian.net/browse/TASK-113O)
- **Historia:** [VELA-597](https://velalang.atlassian.net/browse/VELA-597)
- **Dependencias:** TASK-113N (structured logging)