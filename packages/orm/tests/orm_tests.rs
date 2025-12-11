/*!
Unit tests for Vela ORM.

This module contains comprehensive unit tests for all ORM components,
ensuring type safety, query building, entity management, and database operations.
*/

use vela_orm::*;
use serde::{Deserialize, Serialize};

// Test entities
#[derive(Debug, Clone, Entity, Serialize, Deserialize, PartialEq)]
#[entity(table = "users", schema = "test")]
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

    #[one_to_many(entity = "Post", mapped_by = "author_id")]
    pub posts: Vec<Post>,
}

#[derive(Debug, Clone, Entity, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Entity, Serialize, Deserialize, PartialEq)]
#[entity(table = "categories")]
pub struct Category {
    #[id]
    #[column(primary_key = true, generated = true)]
    pub id: i64,

    #[column(nullable = false)]
    pub name: String,

    #[many_to_many(entity = "Post", join_table = "post_categories")]
    pub posts: Vec<Post>,
}

#[cfg(test)]
mod entity_tests {
    use super::*;

    #[test]
    fn test_entity_metadata() {
        let metadata = User::metadata();

        assert_eq!(metadata.table_name, "users");
        assert_eq!(metadata.schema, Some("test".to_string()));
        assert_eq!(metadata.primary_key.name, "id");
        assert!(metadata.fields.contains_key("email"));
        assert!(metadata.relations.contains_key("posts"));
    }

    #[test]
    fn test_field_metadata() {
        let metadata = User::metadata();
        let email_field = metadata.fields.get("email").unwrap();

        assert_eq!(email_field.name, "email");
        assert_eq!(email_field.column_name, "email");
        assert!(!email_field.nullable);
        assert!(email_field.unique);
        assert!(!email_field.generated);
    }

    #[test]
    fn test_relation_metadata() {
        let metadata = User::metadata();
        let posts_relation = metadata.relations.get("posts").unwrap();

        assert_eq!(posts_relation.target_entity, "Post");
        assert_eq!(posts_relation.mapped_by, Some("author_id".to_string()));
    }
}

#[cfg(test)]
mod query_builder_tests {
    use super::*;
    use crate::config::DatabaseConfig;

    async fn setup_test_db() -> Database {
        let config = DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_basic_query_building() {
        let db = setup_test_db().await;
        let query = User::query(&db)
            .select(&["id", "email", "name"])
            .where_eq("active", true)
            .order_by("name")
            .limit(10);

        let sql = query.build_select_sql();
        assert!(sql.contains("SELECT id, email, name FROM users"));
        assert!(sql.contains("WHERE active = ?"));
        assert!(sql.contains("ORDER BY name ASC"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[tokio::test]
    async fn test_complex_query_with_joins() {
        let db = setup_test_db().await;
        let query = Post::query(&db)
            .join("users", "posts.author_id = users.id")
            .left_join("categories", "posts.id = post_categories.post_id")
            .where_eq("users.active", true)
            .where_like("posts.title", "%test%")
            .group_by(&["posts.id"])
            .having_eq("COUNT(categories.id)", 1)
            .order_by_desc("posts.created_at");

        let sql = query.build_select_sql();
        assert!(sql.contains("INNER JOIN users"));
        assert!(sql.contains("LEFT JOIN categories"));
        assert!(sql.contains("GROUP BY posts.id"));
        assert!(sql.contains("HAVING COUNT(categories.id) = ?"));
        assert!(sql.contains("ORDER BY posts.created_at DESC"));
    }

    #[tokio::test]
    async fn test_where_conditions() {
        let db = setup_test_db().await;

        // Test various where conditions
        let query = User::query(&db)
            .where_eq("status", "active")
            .where_ne("role", "admin")
            .where_gt("age", 18)
            .where_gte("score", 80.5)
            .where_lt("login_count", 100)
            .where_lte("rating", 5.0)
            .where_like("name", "John%")
            .where_in("id", &[1i64, 2, 3])
            .where_null("deleted_at")
            .where_not_null("verified_at");

        let sql = query.build_select_sql();
        assert!(sql.contains("status = ?"));
        assert!(sql.contains("role != ?"));
        assert!(sql.contains("age > ?"));
        assert!(sql.contains("score >= ?"));
        assert!(sql.contains("login_count < ?"));
        assert!(sql.contains("rating <= ?"));
        assert!(sql.contains("name LIKE ?"));
        assert!(sql.contains("id IN (?, ?, ?)"));
        assert!(sql.contains("deleted_at IS NULL"));
        assert!(sql.contains("verified_at IS NOT NULL"));
    }

    #[tokio::test]
    async fn test_query_count_and_exists() {
        let db = setup_test_db().await;

        let count_query = User::query(&db)
            .where_eq("active", true);

        // Test count
        let count = count_query.count().await;
        assert!(count.is_ok()); // Would be 0 for empty table

        // Test exists
        let exists = count_query.exists().await;
        assert!(exists.is_ok()); // Would be false for empty table
    }
}

#[cfg(test)]
mod entity_manager_tests {
    use super::*;
    use crate::entity::EntityManager;

    #[test]
    fn test_entity_manager_sql_generation() {
        let manager = EntityManager::new();

        // Test INSERT SQL generation
        let insert_sql = manager.generate_insert_sql(&User {
            id: 0,
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            created_at: chrono::Utc::now(),
            posts: vec![],
        });

        assert!(insert_sql.is_ok());
        let sql = insert_sql.unwrap();
        assert!(sql.contains("INSERT INTO test.users"));
        assert!(sql.contains("email, name, created_at"));
        assert!(sql.contains("?, ?, ?"));

        // Test UPDATE SQL generation
        let update_sql = manager.generate_update_sql(&User {
            id: 1,
            email: "updated@example.com".to_string(),
            name: "Updated User".to_string(),
            created_at: chrono::Utc::now(),
            posts: vec![],
        });

        assert!(update_sql.is_ok());
        let sql = update_sql.unwrap();
        assert!(sql.contains("UPDATE test.users SET"));
        assert!(sql.contains("email = ?, name = ?, created_at = ?"));
        assert!(sql.contains("WHERE id = ?"));
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    use crate::transaction::{TransactionManager, IsolationLevel, TransactionOptions};

    async fn setup_test_db() -> Database {
        let config = crate::config::DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_transaction_manager() {
        let db = setup_test_db().await;
        let manager = TransactionManager::new(db);

        let result = manager.execute(|tx| async move {
            // This would execute within a transaction
            // For testing, we'll just return a value
            Ok(42)
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_transaction_with_isolation() {
        let db = setup_test_db().await;
        let manager = TransactionManager::new(db);

        let result = manager.execute_with_isolation(
            IsolationLevel::Serializable,
            |tx| async move {
                Ok("success")
            }
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_readonly_transaction() {
        let db = setup_test_db().await;
        let manager = TransactionManager::new(db);

        let result = manager.execute_readonly(|tx| async move {
            Ok("readonly")
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "readonly");
    }

    #[tokio::test]
    async fn test_enhanced_transaction_savepoints() {
        let db = setup_test_db().await;
        let mut tx = crate::transaction::EnhancedTransaction::new(db).await.unwrap();

        // Create savepoints
        tx.savepoint("point1").await.unwrap();
        tx.savepoint("point2").await.unwrap();

        assert_eq!(tx.savepoints(), &["point1".to_string(), "point2".to_string()]);

        // Rollback to savepoint
        tx.rollback_to_savepoint("point1").await.unwrap();
        assert_eq!(tx.savepoints(), &["point1".to_string()]);

        // Release savepoint
        tx.release_savepoint("point1").await.unwrap();
        assert!(tx.savepoints().is_empty());

        // Commit
        tx.commit().await.unwrap();
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use crate::migration::{MigrationRunner, MigrationGenerator, MigrationStatus};
    use std::fs;

    // Test migration implementation
    pub struct CreateUsersTable;

    #[async_trait::async_trait]
    impl crate::migration::Migration for CreateUsersTable {
        fn version(&self) -> &str { "001" }
        fn name(&self) -> &str { "create_users_table" }
        fn description(&self) -> &str { "Create the users table" }

        async fn up(&self, db: &Database) -> Result<()> {
            db.execute(
                "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL, name TEXT NOT NULL)",
                &[]
            ).await?;
            Ok(())
        }

        async fn down(&self, db: &Database) -> Result<()> {
            db.execute("DROP TABLE users", &[]).await?;
            Ok(())
        }
    }

    async fn setup_test_db() -> Database {
        let config = crate::config::DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_migration_runner() {
        let db = setup_test_db().await;
        let runner = MigrationRunner::new(db);

        // Initialize migrations
        runner.initialize().await.unwrap();

        // Get applied migrations (should be empty)
        let applied = runner.get_applied_migrations().await.unwrap();
        assert!(applied.is_empty());

        // Create test migration
        let migration: Box<dyn crate::migration::Migration> = Box::new(CreateUsersTable);

        // Run migration
        runner.migrate(&[migration]).await.unwrap();

        // Check that migration was recorded
        let applied = runner.get_applied_migrations().await.unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].version, "001");
        assert_eq!(applied[0].name, "create_users_table");
    }

    #[test]
    fn test_migration_generator() {
        let temp_dir = "temp_migrations";
        let generator = MigrationGenerator::new(temp_dir);

        // Generate migration
        let version = generator.generate_migration("create_posts_table").unwrap();

        // Check that file was created
        let expected_file = format!("{}/create_posts_table.rs", temp_dir);
        assert!(fs::metadata(&expected_file).is_ok());

        // Read and verify content
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("CreatePostsTable"));
        assert!(content.contains("create_posts_table"));
        assert!(content.contains("CREATE TABLE"));

        // Cleanup
        fs::remove_file(expected_file).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_migration_status() {
        let status = MigrationStatus {
            applied_migrations: vec![],
            pending_migrations: vec!["001_create_users".to_string(), "002_create_posts".to_string()],
        };

        assert!(!status.is_up_to_date());
        assert_eq!(status.applied_count(), 0);
        assert_eq!(status.pending_count(), 2);
    }
}

#[cfg(test)]
mod relation_tests {
    use super::*;
    use crate::relations::{RelationLoader, CascadeManager, LazyLoader};

    async fn setup_test_db() -> Database {
        let config = crate::config::DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[test]
    fn test_relation_loader_creation() {
        let db = setup_test_db().await;
        let loader = RelationLoader::new(db);

        // Loader should be created successfully
        // In a real test, we'd populate data and test loading
    }

    #[test]
    fn test_cascade_manager() {
        let db = setup_test_db().await;
        let manager = CascadeManager::new(db);

        // Manager should be created successfully
        // In a real test, we'd test cascade operations
    }

    #[tokio::test]
    async fn test_lazy_loader() {
        let db = setup_test_db().await;
        let relation_meta = crate::entity::RelationMetadata::one_to_many("Post", "author_id");
        let loader = LazyLoader::<User>::new(db, 1, relation_meta);

        assert_eq!(loader.entity_id, 1);
        // In a real test, we'd call load() and verify results
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use crate::error::{Error, ValidationError, ValidationErrors};

    #[test]
    fn test_error_creation() {
        let err = Error::connection("Failed to connect");
        assert!(err.is_connection());

        let err = Error::validation("Invalid data");
        assert!(err.is_validation());

        let err = Error::entity("Entity not found");
        assert!(err.is_entity());
    }

    #[test]
    fn test_validation_errors() {
        let mut errors = ValidationErrors::new();

        errors.add(ValidationError {
            field: "email".to_string(),
            rule: "email".to_string(),
            message: "Invalid email format".to_string(),
            value: Some("invalid-email".to_string()),
        });

        errors.add(ValidationError {
            field: "name".to_string(),
            rule: "required".to_string(),
            message: "Name is required".to_string(),
            value: None,
        });

        assert_eq!(errors.len(), 2);
        assert!(!errors.is_empty());

        let orm_error = errors.into_orm_error();
        assert!(orm_error.is_validation());
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use crate::config::{DatabaseConfig, DatabaseDriver};

    #[test]
    fn test_database_config_creation() {
        let config = DatabaseConfig::postgres("localhost", 5432, "testdb", "user", "pass");
        assert!(matches!(config.driver, DatabaseDriver::Postgres));
        assert_eq!(config.database, "testdb");

        let config = DatabaseConfig::sqlite(":memory:");
        assert!(matches!(config.driver, DatabaseDriver::Sqlite));
        assert_eq!(config.database, ":memory:");
    }

    #[test]
    fn test_config_url_conversion() {
        let config = DatabaseConfig::postgres("localhost", 5432, "testdb", "user", "pass");
        let url = config.to_url();
        assert_eq!(url, "postgres://user:pass@localhost:5432/testdb");

        // Test parsing URL back
        let parsed = DatabaseConfig::from_url(&url).unwrap();
        assert!(matches!(parsed.driver, DatabaseDriver::Postgres));
        assert_eq!(parsed.database, "testdb");
    }
}

// Integration test that requires a real database
#[cfg(feature = "integration_tests")]
mod integration_tests {
    use super::*;

    async fn setup_postgres_db() -> Database {
        let config = DatabaseConfig::postgres("localhost", 5432, "vela_orm_test", "postgres", "password");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_full_crud_operations() {
        let db = setup_postgres_db().await;

        // Create tables
        db.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            )",
            &[]
        ).await.unwrap();

        // Create a user
        let user = User {
            id: 0, // Will be set by database
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            created_at: chrono::Utc::now(),
            posts: vec![],
        };

        // Save user
        user.save(&db).await.unwrap();

        // Find user
        let found_user = User::find_by_id(&db, 1).await.unwrap().unwrap();
        assert_eq!(found_user.email, "test@example.com");
        assert_eq!(found_user.name, "Test User");

        // Update user
        let mut updated_user = found_user.clone();
        updated_user.name = "Updated User".to_string();
        updated_user.save(&db).await.unwrap();

        // Verify update
        let updated_found = User::find_by_id(&db, 1).await.unwrap().unwrap();
        assert_eq!(updated_found.name, "Updated User");

        // Delete user
        updated_found.delete(&db).await.unwrap();

        // Verify deletion
        let deleted_user = User::find_by_id(&db, 1).await.unwrap();
        assert!(deleted_user.is_none());

        // Cleanup
        db.execute("DROP TABLE users", &[]).await.unwrap();
    }

    #[tokio::test]
    async fn test_query_operations() {
        let db = setup_postgres_db().await;

        // Create and populate test data
        db.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            )",
            &[]
        ).await.unwrap();

        // Insert test data
        for i in 1..=10 {
            db.execute(
                "INSERT INTO users (email, name, active) VALUES (?, ?, ?)",
                &[&format!("user{}@example.com", i), &format!("User {}", i), &(i % 2 == 0)]
            ).await.unwrap();
        }

        // Test queries
        let all_users = User::find_all(&db).await.unwrap();
        assert_eq!(all_users.len(), 10);

        let active_users = User::query(&db)
            .where_eq("active", true)
            .find_many()
            .await
            .unwrap();
        assert_eq!(active_users.len(), 5);

        let user_count = User::query(&db)
            .where_like("name", "User 1%")
            .count()
            .await
            .unwrap();
        assert_eq!(user_count, 2); // User 1, User 10

        // Cleanup
        db.execute("DROP TABLE users", &[]).await.unwrap();
    }
}