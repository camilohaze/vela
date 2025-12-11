# Vela ORM

Type-safe Object-Relational Mapping for the Vela programming language.

[![Crates.io](https://img.shields.io/crates/v/vela-orm.svg)](https://crates.io/crates/vela-orm)
[![Documentation](https://docs.rs/vela-orm/badge.svg)](https://docs.rs/vela-orm)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/velalang/vela)

## Features

- **Type-safe query building** with compile-time validation
- **Automatic entity mapping** with zero boilerplate
- **Multiple database support** (PostgreSQL, MySQL, SQLite)
- **Relation handling** (one-to-one, one-to-many, many-to-many)
- **Migration system** with automatic schema versioning
- **Connection pooling** with health monitoring
- **Transaction management** with savepoints and isolation levels
- **Lazy and eager loading** for optimal performance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vela-orm = "0.1"
```

For specific database drivers:

```toml
[dependencies]
vela-orm = { version = "0.1", features = ["postgres", "mysql"] }
```

Available features:
- `postgres` - PostgreSQL support
- `mysql` - MySQL support
- `sqlite` - SQLite support
- `full` - All database drivers

## Quick Start

### 1. Define Entities

```rust
use vela_orm::{Entity, Column, Id, OneToMany, ManyToOne};
use serde::{Serialize, Deserialize};

#[derive(Entity, Serialize, Deserialize)]
#[entity(table = "users", schema = "public")]
pub struct User {
    #[id]
    #[column(primary_key = true, generated = true)]
    pub id: i64,

    #[column(nullable = false, unique = true)]
    pub email: String,

    #[column(nullable = false)]
    pub name: String,

    #[column(nullable = false, default = "now()")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    // Relations
    #[one_to_many(entity = "Post", mapped_by = "author_id")]
    pub posts: Vec<Post>,
}

#[derive(Entity, Serialize, Deserialize)]
#[entity(table = "posts")]
pub struct Post {
    #[id]
    #[column(primary_key = true, generated = true)]
    pub id: i64,

    #[column(nullable = false)]
    pub title: String,

    #[column(nullable = false)]
    pub content: String,

    #[many_to_one(entity = "User", join_column = "author_id")]
    pub author: User,
}
```

### 2. Connect to Database

```rust
use vela_orm::{Database, DatabaseConfig};

// PostgreSQL
let config = DatabaseConfig::postgres(
    "localhost",
    5432,
    "myapp",
    "username",
    "password"
);

// MySQL
let config = DatabaseConfig::mysql(
    "localhost",
    3306,
    "myapp",
    "username",
    "password"
);

// SQLite
let config = DatabaseConfig::sqlite("./app.db");

let db = Database::connect_with_config(config).await?;
```

### 3. Use the ORM

```rust
// Create entities
let user = User {
    id: 0, // Will be auto-generated
    email: "john@example.com".to_string(),
    name: "John Doe".to_string(),
    created_at: chrono::Utc::now(),
    posts: vec![],
};

// Save to database
user.save(&db).await?;

// Query entities
let user = User::find_by_id(&db, 1).await?.unwrap();
let all_users = User::find_all(&db).await?;

// Type-safe queries
let active_users = User::query(&db)
    .where_eq("active", true)
    .where_like("name", "John%")
    .order_by("name")
    .limit(10)
    .find_many()
    .await?;

// Complex queries with joins
let users_with_posts = User::query(&db)
    .with("posts")  // Eager load posts
    .join("department")
    .where("department.name", "Engineering")
    .find_many()
    .await?;
```

## Advanced Usage

### Transactions

```rust
use vela_orm::transaction::{TransactionManager, IsolationLevel};

// Automatic transaction management
let manager = TransactionManager::new(db.clone());
let result = manager.execute(|tx| async move {
    let user = User::create(user_data, tx).await?;
    let post = Post::create(post_data, tx).await?;
    Ok((user, post))
}).await?;

// Custom isolation level
let result = manager.execute_with_isolation(
    IsolationLevel::Serializable,
    |tx| async move {
        // Critical operation
        Ok(())
    }
).await?;
```

### Migrations

```rust
use vela_orm::migration::{MigrationRunner, Migration};

// Define migration
pub struct CreateUsersTable;

#[async_trait::async_trait]
impl Migration for CreateUsersTable {
    fn version(&self) -> &str { "001" }
    fn name(&self) -> &str { "create_users_table" }

    async fn up(&self, db: &Database) -> Result<()> {
        db.execute(
            "CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL
            )",
            &[]
        ).await?;
        Ok(())
    }

    async fn down(&self, db: &Database) -> Result<()> {
        db.execute("DROP TABLE users", &[]).await?;
        Ok(())
    }
}

// Run migrations
let runner = MigrationRunner::new(db);
let migrations: Vec<Box<dyn Migration>> = vec![Box::new(CreateUsersTable)];
runner.migrate(&migrations).await?;
```

### Relations

```rust
// Lazy loading (default)
let user = User::find_by_id(&db, 1).await?;
let posts = user.posts; // Loads when accessed

// Eager loading
let user_with_posts = User::query(&db)
    .with("posts") // Load posts immediately
    .find_by_id(1)
    .await?;

// Many-to-many relations
#[derive(Entity)]
#[entity(table = "categories")]
pub struct Category {
    #[id]
    pub id: i64,

    #[column]
    pub name: String,

    #[many_to_many(entity = "Post", join_table = "post_categories")]
    pub posts: Vec<Post>,
}
```

### Custom Query Building

```rust
// Raw SQL with type safety
let users = User::query(&db)
    .select(&["id", "email", "COUNT(posts.id) as post_count"])
    .join("posts", "users.id = posts.author_id")
    .where("users.created_at", ">", "2024-01-01")
    .group_by(&["users.id"])
    .having("COUNT(posts.id)", ">", 5)
    .order_by_desc("post_count")
    .find_many()
    .await?;
```

## Configuration

### Database Configuration

```rust
use vela_orm::DatabaseConfig;

let config = DatabaseConfig {
    driver: DatabaseDriver::Postgres,
    host: Some("localhost".to_string()),
    port: Some(5432),
    database: "myapp".to_string(),
    username: Some("user".to_string()),
    password: Some("pass".to_string()),
    pool: ConnectionPoolConfig {
        max_connections: 20,
        min_connections: 5,
        connection_timeout: Duration::from_secs(30),
        max_lifetime: Duration::from_secs(30 * 60),
        idle_timeout: Duration::from_secs(10 * 60),
        health_check_interval: Duration::from_secs(30),
    },
    ssl: SslConfig {
        enabled: true,
        mode: SslMode::Require,
        ca_cert: None,
        client_cert: None,
        client_key: None,
        accept_invalid_certs: false,
    },
    options: HashMap::new(),
};
```

### Entity Configuration

```rust
#[derive(Entity)]
#[entity(
    table = "users",
    schema = "public"
)]
pub struct User {
    #[id]
    #[column(
        primary_key = true,
        generated = true,
        sql_type = "BIGSERIAL"
    )]
    pub id: i64,

    #[column(
        nullable = false,
        unique = true,
        sql_type = "VARCHAR(255)"
    )]
    pub email: String,

    #[column(
        nullable = false,
        default = "CURRENT_TIMESTAMP",
        sql_type = "TIMESTAMP"
    )]
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

## Performance Optimization

### Connection Pooling

The ORM automatically manages connection pools with configurable settings:

- **Max connections**: Maximum number of connections
- **Min connections**: Minimum number of connections to maintain
- **Connection timeout**: Maximum time to wait for a connection
- **Max lifetime**: Maximum lifetime of a connection
- **Idle timeout**: Maximum idle time before closing connection
- **Health checks**: Regular connection health monitoring

### Query Optimization

- **Prepared statements**: Automatic reuse of query plans
- **Connection pooling**: Efficient connection management
- **Lazy loading**: Load relations only when needed
- **Eager loading**: Load related data in single queries
- **Query batching**: Group similar queries

### N+1 Query Prevention

```rust
// ❌ N+1 Problem
let users = User::find_all(&db).await?;
for user in users {
    let posts = user.posts; // Separate query for each user
}

// ✅ Solution: Eager loading
let users_with_posts = User::query(&db)
    .with("posts") // Single query with JOIN
    .find_all()
    .await?;
```

## Error Handling

The ORM provides comprehensive error handling:

```rust
use vela_orm::error::{Error, Result};

match User::find_by_id(&db, 999).await {
    Ok(Some(user)) => println!("Found user: {}", user.name),
    Ok(None) => println!("User not found"),
    Err(Error::Connection(msg)) => eprintln!("Connection error: {}", msg),
    Err(Error::Query(msg)) => eprintln!("Query error: {}", msg),
    Err(Error::Entity(msg)) => eprintln!("Entity error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vela_orm::DatabaseConfig;

    async fn setup_test_db() -> Database {
        let config = DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_user_creation() {
        let db = setup_test_db().await;

        let user = User {
            id: 0,
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            created_at: chrono::Utc::now(),
            posts: vec![],
        };

        user.save(&db).await.unwrap();

        let found = User::find_by_id(&db, 1).await.unwrap().unwrap();
        assert_eq!(found.email, "test@example.com");
    }
}
```

## Contributing

Contributions are welcome! Please see the [contributing guide](../../CONTRIBUTING.md) for details.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This ORM is inspired by:
- **TypeORM** - Type-safe ORM for TypeScript/JavaScript
- **Diesel** - Safe, extensible ORM and Query Builder for Rust
- **SQLAlchemy** - Python SQL toolkit and Object Relational Mapper
- **Hibernate** - Object-relational mapping tool for Java