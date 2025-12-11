/*!
# Vela ORM

Type-safe Object-Relational Mapping for the Vela programming language.

This crate provides a comprehensive ORM solution with:
- Type-safe query building
- Automatic entity mapping
- Relation handling (one-to-one, one-to-many, many-to-many)
- Migration system
- Connection pooling
- Multiple database support (PostgreSQL, MySQL, SQLite)

## Architecture

The ORM is structured in layers:

1. **Entity Layer**: Entity definitions and metadata
2. **Query Layer**: Type-safe query building and execution
3. **Relations Layer**: Handling of entity relationships
4. **Migrations Layer**: Database schema versioning
5. **Connection Layer**: Connection pooling and transaction management

## Example

```rust,no_run
use vela_orm::{Database, Entity, Column, Id, OneToMany, ManyToOne};
use serde::{Serialize, Deserialize};

#[derive(Entity, Serialize, Deserialize)]
#[entity(table = "users")]
pub struct User {
    #[id]
    #[column(primary_key = true, generated = true)]
    pub id: i64,

    #[column(nullable = false)]
    pub name: String,

    #[column(nullable = false, unique = true)]
    pub email: String,

    #[one_to_many(entity = "Post", mapped_by = "author")]
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

// Connect to database
let db = Database::connect("postgres://user:pass@localhost/myapp").await?;

// Query entities
let user = User::find_by_id(&db, 1).await?;
let posts = Post::query(&db)
    .where_eq("author_id", user.id)
    .find_many()
    .await?;
```
*/

pub mod config;
pub mod connection;
pub mod entity;
pub mod error;
pub mod migration;
pub mod query;
pub mod relations;
pub mod transaction;

// Re-exports for convenience
pub use config::DatabaseConfig;
pub use connection::Database;
pub use entity::{Entity, EntityManager};
pub use error::{Result, Error};
pub use query::QueryBuilder;

/// Version of the Vela ORM
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the ORM with default settings
pub async fn init() -> Result<()> {
    tracing::info!("Initializing Vela ORM v{}", VERSION);
    Ok(())
}

/// Initialize the ORM with custom configuration
pub async fn init_with_config(config: DatabaseConfig) -> Result<Database> {
    tracing::info!("Initializing Vela ORM v{} with custom config", VERSION);
    Database::connect_with_config(config).await
}