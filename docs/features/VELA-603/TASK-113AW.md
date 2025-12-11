# TASK-113AW: Diseñar Arquitectura de ORM

## 📋 Información General
- **Historia:** VELA-603
- **Estado:** Finalizada ✅
- **Fecha:** 2025-12-11
- **Tipo:** Arquitectura / Diseño

## 🎯 Objetivo
Diseñar la arquitectura completa del sistema ORM type-safe para Vela, incluyendo:
- Arquitectura en capas del ORM
- APIs de alto nivel para desarrolladores
- Soporte para múltiples bases de datos
- Estrategias de optimización de performance

## 🔨 Implementación

### Arquitectura en Capas

El ORM se estructura en 5 capas principales:

#### 1. Entity Layer (Capa de Entidades)
**Responsabilidades:**
- Definición de entidades con decoradores
- Metadata de mapeo entidad-tabla
- Validación de constraints
- Type safety en propiedades

**Componentes:**
- `EntityManager`: Gestión del ciclo de vida de entidades
- `EntityMetadata`: Metadata de mapeo
- `EntityValidator`: Validación de entidades

#### 2. Query Layer (Capa de Queries)
**Responsabilidades:**
- Construcción de queries type-safe
- Compilación a SQL nativo
- Ejecución optimizada
- Result mapping automático

**Componentes:**
- `QueryBuilder`: API fluida para queries
- `SqlCompiler`: Compilación a SQL
- `ResultMapper`: Mapeo de resultados a objetos

#### 3. Relations Layer (Capa de Relaciones)
**Responsabilidades:**
- Manejo de relaciones entre entidades
- Lazy vs eager loading
- Operaciones en cascada
- Resolución de dependencias

**Componentes:**
- `RelationLoader`: Carga de relaciones
- `CascadeManager`: Operaciones en cascada
- `RelationResolver`: Resolución de claves foráneas

#### 4. Migrations Layer (Capa de Migraciones)
**Responsabilidades:**
- Versionado de schema de base de datos
- Aplicación y rollback de cambios
- Tracking del estado del schema

**Componentes:**
- `MigrationRunner`: Ejecución de migraciones
- `SchemaTracker`: Estado del schema
- `MigrationGenerator`: Generación automática

#### 5. Connection Layer (Capa de Conexión)
**Responsabilidades:**
- Pool de conexiones
- Gestión de transacciones
- Health monitoring
- Configuración de conexión

**Componentes:**
- `ConnectionPool`: Pool de conexiones
- `TransactionManager`: Gestión de transacciones
- `HealthChecker`: Monitoreo de salud

### API de Alto Nivel

#### Definición de Entidades
```vela
@entity(table: "users", schema: "public")
class User {
    @id
    @column(type: "uuid", primary_key: true, generated: true)
    id: String

    @column(type: "varchar(255)", nullable: false, unique: false)
    name: String

    @column(type: "varchar(255)", nullable: false, unique: true)
    email: String

    @column(type: "timestamp", default: "now()", nullable: false)
    created_at: DateTime

    @column(type: "timestamp", nullable: true)
    updated_at: DateTime

    // Relations
    @oneToMany(entity: Post, mapped_by: "author")
    posts: List<Post>

    @manyToOne(entity: Department, join_column: "department_id")
    department: Department
}
```

#### Query Builder Type-Safe
```vela
// Queries básicas
let user = await User.findById(123)
let users = await User.findAll()

// Queries con condiciones
let activeUsers = await User.query()
    .where("active", true)
    .where("created_at", ">", "2024-01-01")
    .orderBy("name")
    .limit(10)
    .findMany()

// Queries con joins
let usersWithPosts = await User.query()
    .with("posts")
    .with("department")
    .where("department.name", "Engineering")
    .findMany()

// Queries complejas
let complexQuery = await User.query()
    .select("name", "email", "department.name")
    .join("department")
    .where("department.budget", ">", 100000)
    .groupBy("department.id")
    .having("count(*)", ">", 5)
    .orderBy("department.name")
    .findMany()
```

#### Operaciones CRUD
```vela
// Create
let newUser = User {
    name: "John Doe",
    email: "john@example.com",
    department_id: dept.id
}
await User.save(newUser)

// Read
let user = await User.findById(123)
let users = await User.query().where("active", true).findMany()

// Update
user.name = "Jane Doe"
await User.save(user)

// Delete
await User.delete(user.id)
// o con soft delete
await User.softDelete(user.id)
```

#### Transacciones
```vela
await Database.transaction(async (tx) => {
    let user = await User.create({ name: "John", email: "john@test.com" }, tx)
    let post = await Post.create({ title: "Hello", author_id: user.id }, tx)

    // Si algo falla, todo se rollback automáticamente
    return { user, post }
})
```

### Soporte de Bases de Datos

#### PostgreSQL (Primario)
```vela
@config
class DatabaseConfig {
    driver: "postgres"
    host: "localhost"
    port: 5432
    database: "myapp"
    username: "user"
    password: "pass"
    ssl_mode: "require"
    max_connections: 20
    min_connections: 5
    connection_timeout: 30
}
```

#### MySQL
```vela
@config
class DatabaseConfig {
    driver: "mysql"
    host: "localhost"
    port: 3306
    database: "myapp"
    username: "user"
    password: "pass"
    charset: "utf8mb4"
}
```

#### SQLite (Desarrollo)
```vela
@config
class DatabaseConfig {
    driver: "sqlite"
    path: "./dev.db"
    foreign_keys: true
    journal_mode: "WAL"
}
```

### Optimizaciones de Performance

#### 1. Query Optimization
- **Prepared Statements**: Reutilización automática
- **Connection Pooling**: Gestión eficiente de conexiones
- **Query Batching**: Agrupación de queries similares
- **Result Caching**: Cache de resultados frecuentes

#### 2. Lazy Loading vs Eager Loading
```vela
// Lazy loading (por defecto)
let user = await User.findById(123)
// posts se cargan solo cuando se acceden
let posts = user.posts // Query ejecutada aquí

// Eager loading
let userWithPosts = await User.query()
    .with("posts")  // Carga en la misma query
    .findById(123)
```

#### 3. N+1 Query Prevention
```vela
// ❌ N+1 Problem
let users = await User.findAll()
for user in users {
    let posts = await user.posts // N queries adicionales
}

// ✅ Solución con eager loading
let usersWithPosts = await User.query()
    .with("posts")  // Una sola query con JOIN
    .findAll()
```

### Seguridad y Validación

#### SQL Injection Prevention
- **Prepared Statements**: Todos los queries usan prepared statements
- **Parameter Binding**: Parámetros tipados automáticamente
- **Query Building**: API segura contra inyección

#### Data Validation
```vela
@entity(table: "users")
class User {
    @column(type: "varchar(255)")
    @validate(min_length: 2, max_length: 100)
    name: String

    @column(type: "varchar(255)")
    @validate(email: true, required: true)
    email: String

    @column(type: "int")
    @validate(min: 0, max: 150)
    age: Number
}

// Validación automática al guardar
await User.save(user) // Lanza error si validation falla
```

## ✅ Criterios de Aceptación
- [x] Arquitectura en capas definida
- [x] APIs de alto nivel diseñadas
- [x] Soporte multi-base de datos especificado
- [x] Estrategias de optimización definidas
- [x] ADR creado en `docs/architecture/`
- [x] Documentación completa generada

## 🔗 Referencias
- **Jira:** [TASK-113AW](https://velalang.atlassian.net/browse/TASK-113AW)
- **Historia:** [VELA-603](https://velalang.atlassian.net/browse/VELA-603)
- **ADR:** `docs/architecture/ADR-113AW-orm-architecture.md`