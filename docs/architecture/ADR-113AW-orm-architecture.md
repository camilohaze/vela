# ADR-113AW: Arquitectura del Sistema ORM Type-Safe

## Estado
✅ Aceptado

## Fecha
2025-12-11

## Contexto
Vela necesita un ORM (Object-Relational Mapping) type-safe que permita a los desarrolladores interactuar con bases de datos de manera segura y expresiva. El ORM debe:

- Proporcionar type safety completo en queries y resultados
- Soportar decoradores para definición de entidades
- Implementar un query builder fluido con autocompletado
- Manejar relaciones entre entidades automáticamente
- Gestionar migraciones de schema de forma segura
- Proporcionar connection pooling eficiente

## Decisión
Implementar un ORM modular con arquitectura en capas que incluye:

### 1. Entity Layer
- **Decoradores**: `@entity`, `@id`, `@column` para definición de entidades
- **Metadata**: Información de mapeo almacenada en runtime
- **Validation**: Validación automática de constraints

### 2. Query Builder Layer
- **Fluent API**: Interface fluida para construcción de queries
- **Type Safety**: Autocompletado y validación de tipos
- **Composition**: Queries composables y reutilizables

### 3. Relations Layer
- **Lazy Loading**: Carga diferida de relaciones
- **Eager Loading**: Carga anticipada con `with()`
- **Cascading**: Operaciones en cascada (save, delete)

### 4. Migrations Layer
- **Versioning**: Control de versiones de schema
- **Rollback**: Reversión segura de cambios
- **State Tracking**: Estado actual del schema

### 5. Connection Layer
- **Pooling**: Pool de conexiones configurables
- **Transactions**: Soporte para transacciones ACID
- **Health Checks**: Monitoreo de conexiones

## Consecuencias

### Positivas
- **Type Safety**: Prevención de errores en tiempo de compilación
- **Developer Experience**: Autocompletado y validación en IDE
- **Performance**: Optimización automática de queries
- **Maintainability**: Código más legible y mantenible
- **Flexibility**: Soporte para múltiples bases de datos

### Negativas
- **Complexity**: Mayor complejidad en la implementación
- **Learning Curve**: Nuevos conceptos para aprender
- **Runtime Overhead**: Costo de metadata y validaciones
- **Database Coupling**: Dependencia de características específicas de DB

## Alternativas Consideradas

### 1. Code Generation Approach
**Descripción**: Generar código Rust/Vela desde schema de DB
**Pros**: Type safety máximo, performance óptima
**Cons**: Complejo de mantener, no flexible para cambios dinámicos
**Rechazada porque**: Menos flexible para desarrollo ágil

### 2. Dynamic ORM (como Prisma)
**Descripción**: ORM completamente dinámico sin generación de código
**Pros**: Flexible, fácil de usar
**Cons**: Pérdida de type safety, errores en runtime
**Rechazada porque**: No cumple con el objetivo de type safety

### 3. Hybrid Approach (Elegido)
**Descripción**: Decoradores + runtime metadata + code generation limitada
**Pros**: Balance entre type safety y flexibilidad
**Cons**: Mayor complejidad de implementación
**Aceptada porque**: Mejor balance para el caso de uso de Vela

## Implementación

### Arquitectura Técnica

```
packages/orm/
├── entity/           # Sistema de entidades
│   ├── decorators.rs # @entity, @id, @column
│   ├── metadata.rs   # Metadata de entidades
│   └── validation.rs # Validación de entidades
├── query/            # Query builder
│   ├── builder.rs    # API fluida
│   ├── compiler.rs   # Compilación a SQL
│   └── executor.rs   # Ejecución de queries
├── relations/        # Sistema de relaciones
│   ├── loader.rs     # Lazy/eager loading
│   ├── cascading.rs  # Operaciones en cascada
│   └── resolver.rs   # Resolución de relaciones
├── migrations/       # Sistema de migraciones
│   ├── runner.rs     # Ejecución de migraciones
│   ├── generator.rs  # Generación automática
│   └── tracker.rs    # Tracking de estado
└── connection/       # Pool de conexiones
    ├── pool.rs       # Connection pooling
    ├── transaction.rs # Transacciones
    └── health.rs     # Health checks
```

### API de Alto Nivel

```vela
// Definición de entidad
@entity(table: "users")
class User {
    @id
    @column(type: "uuid")
    id: String

    @column(type: "varchar(255)")
    name: String

    @oneToMany(entity: Post)
    posts: List<Post>
}

// Uso del ORM
let user = await User.query()
    .where("name", "John")
    .with("posts")
    .findOne()

// Type-safe result
user: Option<User>
```

### Soporte de Bases de Datos

| Base de Datos | Soporte | Driver |
|---------------|---------|--------|
| PostgreSQL    | ✅ Completo | tokio-postgres |
| MySQL         | ✅ Completo | sqlx |
| SQLite        | ✅ Completo | rusqlite |
| MongoDB       | 🚧 Planificado | mongodb |

## Referencias

### Jira
- [VELA-603: ORM Type-Safe](https://velalang.atlassian.net/browse/VELA-603)
- [US-24I: Database Access](https://velalang.atlassian.net/browse/US-24I)

### Documentación Técnica
- [SQL Standard](https://en.wikipedia.org/wiki/SQL)
- [Active Record Pattern](https://en.wikipedia.org/wiki/Active_record_pattern)
- [Data Mapper Pattern](https://en.wikipedia.org/wiki/Data_mapper_pattern)

### Inspiración
- [TypeORM](https://typeorm.io/) - TypeScript ORM
- [Diesel](https://diesel.rs/) - Rust ORM
- [Prisma](https://www.prisma.io/) - Database toolkit
- [SQLAlchemy](https://www.sqlalchemy.org/) - Python ORM

## Implementación
Ver código en: `packages/orm/`
Documentación: `docs/features/VELA-603/`