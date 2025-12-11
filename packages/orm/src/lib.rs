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
use vela_orm::{Database, Entity};
use vela_orm::config::DatabaseConfig;
use serde::{Deserialize, Serialize};

// Define an entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

impl Entity for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn primary_key_field() -> &'static str {
        "id"
    }

    fn metadata() -> vela_orm::entity::EntityMetadata {
        // Implementation of metadata
        vela_orm::entity::EntityMetadata::new("users")
    }

    async fn find_by_id(db: &Database, id: i64) -> vela_orm::Result<Option<Self>> {
        Self::query(db).where_eq("id", id).find_one().await
    }

    async fn find_all(db: &Database) -> vela_orm::Result<Vec<Self>> {
        Self::query(db).find_many().await
    }

    async fn save(&self, db: &Database) -> vela_orm::Result<()> {
        Ok(())
    }

    async fn delete(&self, db: &Database) -> vela_orm::Result<()> {
        Ok(())
    }

    fn query(db: &Database) -> vela_orm::QueryBuilder<Self> {
        vela_orm::QueryBuilder::new(db.clone())
    }
}

// Usage example (in an async function)
async fn example() -> vela_orm::Result<()> {
    // Connect to database
    let config = DatabaseConfig::sqlite(":memory:");
    let db = Database::connect_with_config(config).await?;

    // Query entities
    let user = User::find_by_id(&db, 1).await?;
    let users = User::find_all(&db).await?;

    Ok(())
}
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
pub mod typed_query;

// Re-exports for convenience
pub use config::DatabaseConfig;
pub use connection::Database;
pub use entity::{Entity, EntityManager};
pub use error::{Result, Error};
pub use query::QueryBuilder;
pub use typed_query::{TypedQueryBuilder, Field};

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