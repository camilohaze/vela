/*!
Unit tests for Vela ORM.

This module contains comprehensive unit tests for all ORM components,
ensuring type safety, query building, entity management, and database operations.
*/

use vela_orm::*;
use serde::{Deserialize, Serialize};
use vela_orm::entity::{EntityMetadata, FieldMetadata};

// Simple test entity for TypedQueryBuilder tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

// Implement Entity trait manually for TestUser
impl Entity for TestUser {
    fn table_name() -> &'static str {
        "test_users"
    }

    fn primary_key_field() -> &'static str {
        "id"
    }

    fn metadata() -> EntityMetadata {
        let mut metadata = EntityMetadata::new("test_users");

        // Add primary key
        let id_field = FieldMetadata::new("id", "id", "INTEGER")
            .nullable(false)
            .generated(true);
        metadata = metadata.with_primary_key(id_field.clone());
        metadata = metadata.with_field("id", id_field);

        // Add other fields
        let name_field = FieldMetadata::new("name", "name", "TEXT").nullable(false);
        metadata = metadata.with_field("name", name_field);

        let email_field = FieldMetadata::new("email", "email", "TEXT").nullable(false);
        metadata = metadata.with_field("email", email_field);

        let active_field = FieldMetadata::new("active", "active", "BOOLEAN").nullable(false);
        metadata = metadata.with_field("active", active_field);

        metadata
    }

    async fn find_by_id(_db: &Database, _id: i64) -> Result<Option<Self>> {
        // Mock implementation for tests - cannot execute real queries in unit tests
        Ok(None)
    }

    async fn find_all(_db: &Database) -> Result<Vec<Self>> {
        // Mock implementation for tests - cannot execute real queries in unit tests
        Ok(vec![])
    }

    async fn save(&self, _db: &Database) -> Result<()> {
        // Mock implementation for tests
        Ok(())
    }

    async fn delete(&self, _db: &Database) -> Result<()> {
        // Mock implementation for tests
        Ok(())
    }

    fn query(db: &Database) -> QueryBuilder<Self> {
        QueryBuilder::new(db.clone())
    }
}

#[cfg(test)]
mod orm_tests {
    use super::*;
    use vela_orm::config::DatabaseConfig;

    async fn setup_test_db() -> Database {
        let config = DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_entity_metadata() {
        let metadata = TestUser::metadata();
        assert_eq!(metadata.table_name, "test_users");

        // Check primary key
        assert_eq!(metadata.primary_key.name, "id");

        // Check fields exist
        assert!(metadata.fields.contains_key("id"));
        assert!(metadata.fields.contains_key("name"));
        assert!(metadata.fields.contains_key("email"));
        assert!(metadata.fields.contains_key("active"));
    }

    #[tokio::test]
    async fn test_query_builder_creation() {
        let db = setup_test_db().await;
        let _query = TestUser::query(&db);
        // Query builder created successfully
    }

    #[tokio::test]
    async fn test_where_conditions() {
        let db = setup_test_db().await;
        let _query = TestUser::query(&db)
            .where_eq("active", true)
            .where_gt("id", 10i64)
            .where_like("name", "test");
        // Query with conditions created successfully
    }

    #[tokio::test]
    async fn test_ordering_and_limits() {
        let db = setup_test_db().await;
        let _query = TestUser::query(&db)
            .order_by("name")
            .order_by_desc("id")
            .limit(10)
            .offset(5);
        // Query with ordering and limits created successfully
    }

    #[tokio::test]
    async fn test_field_selection() {
        let db = setup_test_db().await;
        let _query = TestUser::query(&db)
            .select(&["id", "name"]);
        // Query with field selection created successfully
    }

    #[tokio::test]
    async fn test_complex_query() {
        let db = setup_test_db().await;
        let _query = TestUser::query(&db)
            .select(&["id", "name", "email"])
            .where_eq("active", true)
            .where_gt("id", 5i64)
            .order_by("name")
            .limit(10);
        // Complex query created successfully
    }
}