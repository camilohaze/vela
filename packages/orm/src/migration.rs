/*!
Database migrations for Vela ORM.

This module provides schema versioning, migration tracking,
and automatic migration generation for database schema changes.
*/

use crate::connection::{Database, ToSql};
use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Migration record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration version/ID
    pub version: String,
    /// Migration name
    pub name: String,
    /// When the migration was applied
    pub applied_at: DateTime<Utc>,
    /// Migration checksum for integrity
    pub checksum: String,
}

/// Migration trait that all migrations must implement
#[async_trait::async_trait]
pub trait Migration {
    /// Get the migration version
    fn version(&self) -> &str;

    /// Get the migration name
    fn name(&self) -> &str;

    /// Get the migration description
    fn description(&self) -> &str {
        ""
    }

    /// Execute the migration up
    async fn up(&self, db: &Database) -> Result<()>;

    /// Execute the migration down (rollback)
    async fn down(&self, db: &Database) -> Result<()>;
}

/// Migration runner for executing migrations
pub struct MigrationRunner {
    db: Database,
    migrations_table: String,
    migrations_path: String,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(db: Database) -> Self {
        Self {
            db,
            migrations_table: "schema_migrations".to_string(),
            migrations_path: "migrations".to_string(),
        }
    }

    /// Set the migrations table name
    pub fn with_migrations_table(mut self, table: &str) -> Self {
        self.migrations_table = table.to_string();
        self
    }

    /// Set the migrations path
    pub fn with_migrations_path(mut self, path: &str) -> Self {
        self.migrations_path = path.to_string();
        self
    }

    /// Initialize the migrations system
    pub async fn initialize(&self) -> Result<()> {
        // Create migrations table if it doesn't exist
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                version VARCHAR(255) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                checksum VARCHAR(255) NOT NULL
            )
            "#,
            self.migrations_table
        );

        self.db.execute(&sql, vec![]).await?;
        Ok(())
    }

    /// Get all applied migrations
    pub async fn get_applied_migrations(&self) -> Result<Vec<MigrationRecord>> {
        let sql = format!(
            "SELECT version, name, applied_at, checksum FROM {} ORDER BY applied_at",
            self.migrations_table
        );

        let result = self.db.query(&sql, vec![]).await?;
        let mut migrations = Vec::new();

        for row in result.rows {
            let version = row.try_get::<String>(0)?;
            let name = row.try_get::<String>(1)?;
            let applied_at = row.try_get::<DateTime<Utc>>(2)?;
            let checksum = row.try_get::<String>(3)?;

            migrations.push(MigrationRecord {
                version,
                name,
                applied_at,
                checksum,
            });
        }

        Ok(migrations)
    }

    /// Apply pending migrations
    pub async fn migrate<M: Migration>(&self, migrations: &[M]) -> Result<()> {
        self.initialize().await?;

        let applied = self.get_applied_migrations().await?;
        let applied_versions: std::collections::HashSet<String> =
            applied.iter().map(|m| m.version.clone()).collect();

        for migration in migrations.iter() {
            if applied_versions.contains(migration.version()) {
                continue; // Already applied
            }

            tracing::info!("Applying migration: {} - {}", migration.version(), migration.name());

            // Start transaction
            let mut tx = self.db.transaction().await?;

            // Execute migration
            migration.up(&self.db).await?;

            // Record migration
            let checksum = self.calculate_checksum(migration);
            let sql = format!(
                "INSERT INTO {} (version, name, checksum) VALUES (?, ?, ?)",
                self.migrations_table
            );

            tx.execute(&sql, vec![
                Box::new(migration.version().to_string()) as Box<dyn ToSql>,
                Box::new(migration.name().to_string()) as Box<dyn ToSql>,
                Box::new(checksum.clone()) as Box<dyn ToSql>,
            ]).await?;

            // Commit transaction
            tx.commit().await?;

            tracing::info!("Migration applied successfully: {}", migration.version());
        }

        Ok(())
    }

    /// Rollback the last migration
    pub async fn rollback<M: Migration>(&self, migrations: &[&M]) -> Result<()> {
        let applied = self.get_applied_migrations().await?;

        if let Some(last_migration) = applied.last() {
            // Find the migration to rollback
            let migration_to_rollback = migrations
                .iter()
                .find(|m| m.version() == last_migration.version)
                .ok_or_else(|| Error::migration(format!("Migration {} not found", last_migration.version)))?;

            tracing::info!("Rolling back migration: {} - {}", last_migration.version, last_migration.name);

            // Start transaction
            let mut tx = self.db.transaction().await?;

            // Execute rollback
            migration_to_rollback.down(&self.db).await?;

            // Remove migration record
            let sql = format!("DELETE FROM {} WHERE version = ?", self.migrations_table);
            tx.execute(&sql, vec![Box::new(last_migration.version.clone()) as Box<dyn ToSql>]).await?;

            // Commit transaction
            tx.commit().await?;

            tracing::info!("Migration rolled back successfully: {}", last_migration.version);
        } else {
            tracing::info!("No migrations to rollback");
        }

        Ok(())
    }

    /// Rollback to a specific migration version
    pub async fn rollback_to<M: Migration>(&self, target_version: &str, migrations: &[&M]) -> Result<()> {
        let applied = self.get_applied_migrations().await?;

        // Find migrations to rollback (from newest to target)
        let mut to_rollback = Vec::new();
        for migration in applied.iter().rev() {
            if migration.version == target_version {
                break;
            }
            to_rollback.push(migration.version.clone());
        }

        for version in to_rollback {
            let migration = migrations
                .iter()
                .find(|m| m.version() == version)
                .ok_or_else(|| Error::migration(format!("Migration {} not found", version)))?;

            tracing::info!("Rolling back migration: {} - {}", version, migration.name());

            // Start transaction
            let mut tx = self.db.transaction().await?;

            // Execute rollback
            migration.down(&self.db).await?;

            // Remove migration record
            let sql = format!("DELETE FROM {} WHERE version = ?", self.migrations_table);
            tx.execute(&sql, vec![Box::new(version.clone()) as Box<dyn ToSql>]).await?;

            // Commit transaction
            tx.commit().await?;

            tracing::info!("Migration rolled back successfully: {}", version);
        }

        Ok(())
    }

    /// Get the current migration status
    pub async fn status<M: Migration>(&self, migrations: &[&M]) -> Result<MigrationStatus> {
        let applied = self.get_applied_migrations().await?;
        let applied_versions: std::collections::HashSet<String> =
            applied.iter().map(|m| m.version.clone()).collect();

        let mut pending = Vec::new();
        for migration in migrations {
            if !applied_versions.contains(migration.version()) {
                pending.push(migration.version().to_string());
            }
        }

        Ok(MigrationStatus {
            applied_migrations: applied,
            pending_migrations: pending,
        })
    }

    /// Calculate checksum for a migration
    fn calculate_checksum<M: Migration>(&self, migration: &M) -> String {
        // Calculate checksum based on migration version and name
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        migration.version().hash(&mut hasher);
        migration.name().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Migration status information
#[derive(Debug)]
pub struct MigrationStatus {
    /// Applied migrations
    pub applied_migrations: Vec<MigrationRecord>,
    /// Pending migration versions
    pub pending_migrations: Vec<String>,
}

impl MigrationStatus {
    /// Check if all migrations are applied
    pub fn is_up_to_date(&self) -> bool {
        self.pending_migrations.is_empty()
    }

    /// Get the count of applied migrations
    pub fn applied_count(&self) -> usize {
        self.applied_migrations.len()
    }

    /// Get the count of pending migrations
    pub fn pending_count(&self) -> usize {
        self.pending_migrations.len()
    }
}

/// Schema tracker for tracking schema changes
pub struct SchemaTracker {
    db: Database,
    schema_table: String,
}

impl SchemaTracker {
    /// Create a new schema tracker
    pub fn new(db: Database) -> Self {
        Self {
            db,
            schema_table: "schema_tracking".to_string(),
        }
    }

    /// Initialize schema tracking
    pub async fn initialize(&self) -> Result<()> {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                table_name VARCHAR(255) PRIMARY KEY,
                schema_hash VARCHAR(255) NOT NULL,
                last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            self.schema_table
        );

        self.db.execute(&sql, vec![]).await?;
        Ok(())
    }

    /// Track schema changes for a table
    pub async fn track_table_schema(&self, table_name: &str, schema_hash: &str) -> Result<()> {
        let sql = format!(
            r#"
            INSERT INTO {} (table_name, schema_hash, last_updated)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT (table_name) DO UPDATE SET
                schema_hash = EXCLUDED.schema_hash,
                last_updated = CURRENT_TIMESTAMP
            "#,
            self.schema_table
        );

        self.db.execute(&sql, vec![
            Box::new(table_name.to_string()) as Box<dyn ToSql>,
            Box::new(schema_hash.to_string()) as Box<dyn ToSql>,
        ]).await?;
        Ok(())
    }

    /// Get schema hash for a table
    pub async fn get_table_schema_hash(&self, table_name: &str) -> Result<Option<String>> {
        let sql = format!("SELECT schema_hash FROM {} WHERE table_name = ?", self.schema_table);

        let result = self.db.query(&sql, vec![Box::new(table_name.to_string()) as Box<dyn ToSql>]).await?;

        if let Some(row) = result.rows.first() {
            let hash = row.try_get::<String>(0)?;
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }
}

/// Migration generator for creating migration files
pub struct MigrationGenerator {
    migrations_path: String,
}

impl MigrationGenerator {
    /// Create a new migration generator
    pub fn new(migrations_path: &str) -> Self {
        Self {
            migrations_path: migrations_path.to_string(),
        }
    }

    /// Generate a new migration file
    pub fn generate_migration(&self, name: &str) -> Result<String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let version = format!("{}_{}", timestamp, name.to_lowercase().replace(" ", "_"));
        let filename = format!("{}.rs", version);
        let filepath = Path::new(&self.migrations_path).join(filename);

        // Ensure migrations directory exists
        fs::create_dir_all(&self.migrations_path)
            .map_err(|e| Error::migration(format!("Failed to create migrations directory: {}", e)))?;

        // Generate migration template
        let template = format!(
            r#"use vela_orm::{{Database, Migration, Result}};

pub struct {} {{
    version: String,
    name: String,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            version: "{}".to_string(),
            name: "{}".to_string(),
        }}
    }}
}}

#[async_trait::async_trait]
impl Migration for {} {{
    fn version(&self) -> &str {{
        &self.version
    }}

    fn name(&self) -> &str {{
        &self.name
    }}

    fn description(&self) -> &str {{
        "{}"
    }}

    async fn up(&self, db: &Database) -> Result<()> {{
        // TODO: Implement migration up logic
        // Example:
        // db.execute("CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))", &[]).await?;
        Ok(())
    }}

    async fn down(&self, db: &Database) -> Result<()> {{
        // TODO: Implement migration down logic
        // Example:
        // db.execute("DROP TABLE users", &[]).await?;
        Ok(())
    }}
}}
"#,
            self.to_pascal_case(name),
            self.to_pascal_case(name),
            version,
            name,
            self.to_pascal_case(name),
            name
        );

        fs::write(&filepath, template)
            .map_err(|e| Error::migration(format!("Failed to write migration file: {}", e)))?;

        Ok(version)
    }

    /// Convert string to PascalCase
    fn to_pascal_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    async fn test_migration_runner_initialization() {
        let config = DatabaseConfig::sqlite(":memory:");
        let db = Database::connect_with_config(config).await.unwrap();
        let runner = MigrationRunner::new(db);

        // This would normally create the migrations table
        // In a real test, we'd verify the table exists
        assert_eq!(runner.migrations_table, "schema_migrations");
    }

    #[test]
    fn test_migration_generator() {
        let generator = MigrationGenerator::new("test_migrations");

        // This would generate a migration file
        // In a real test, we'd verify the file was created
        let name = generator.to_pascal_case("create_users_table");
        assert_eq!(name, "CreateUsersTable");
    }

    #[test]
    fn test_migration_status() {
        let status = MigrationStatus {
            applied_migrations: vec![],
            pending_migrations: vec!["001_create_users".to_string()],
        };

        assert!(!status.is_up_to_date());
        assert_eq!(status.applied_count(), 0);
        assert_eq!(status.pending_count(), 1);
    }
}