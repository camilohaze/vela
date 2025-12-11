/*!
Unit tests for Vela ORM.

This module contains comprehensive unit tests for all ORM components,
ensuring type safety, query building, entity management, and database operations.
*/

use vela_orm::*;
use serde::{Deserialize, Serialize};

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

    fn id_field() -> &'static str {
        "id"
    }

    fn fields() -> Vec<&'static str> {
        vec!["id", "name", "email", "active"]
    }

    fn metadata() -> EntityMetadata {
        let mut metadata = EntityMetadata::new("test_users");
        metadata.add_field("id", FieldType::Integer, true, true, false);
        metadata.add_field("name", FieldType::String, false, false, false);
        metadata.add_field("email", FieldType::String, false, false, false);
        metadata.add_field("active", FieldType::Boolean, false, false, false);
        metadata
    }

    async fn find_by_id(db: &Database, id: i64) -> Result<Option<Self>> {
        Self::query(db).where_eq(crate::typed_query::id, id).find_one().await
    }

    async fn find_all(db: &Database) -> Result<Vec<Self>> {
        Self::query(db).find_all().await
    }

    async fn save(&self, db: &Database) -> Result<()> {
        // Simple implementation for testing
        Ok(())
    }

    async fn delete(&self, db: &Database) -> Result<()> {
        // Simple implementation for testing
        Ok(())
    }

    fn query(db: &Database) -> TypedQueryBuilder<Self> {
        TypedQueryBuilder::new(db)
    }
}

/*!
Unit tests for Vela ORM.

This module contains comprehensive unit tests for all ORM components,
ensuring type safety, query building, entity management, and database operations.
*/

use vela_orm::*;
use serde::{Deserialize, Serialize};

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

    fn id_field() -> &'static str {
        "id"
    }

    fn fields() -> Vec<&'static str> {
        vec!["id", "name", "email", "active"]
    }

    fn metadata() -> EntityMetadata {
        let mut metadata = EntityMetadata::new("test_users");
        metadata.add_field("id", FieldType::Integer, true, true, false);
        metadata.add_field("name", FieldType::String, false, false, false);
        metadata.add_field("email", FieldType::String, false, false, false);
        metadata.add_field("active", FieldType::Boolean, false, false, false);
        metadata
    }

    async fn find_by_id(db: &Database, id: i64) -> Result<Option<Self>> {
        Self::query(db).where_eq(crate::typed_query::id, id).find_one().await
    }

    async fn find_all(db: &Database) -> Result<Vec<Self>> {
        Self::query(db).find_all().await
    }

    async fn save(&self, db: &Database) -> Result<()> {
        // Simple implementation for testing
        Ok(())
    }

    async fn delete(&self, db: &Database) -> Result<()> {
        // Simple implementation for testing
        Ok(())
    }

    fn query(db: &Database) -> crate::query::QueryBuilder<Self> {
        crate::query::QueryBuilder::new(db.clone())
    }
}

// Generate type-safe field markers for TestUser
entity_fields!(TestUser {
    id: i64,
    name: String,
    email: String,
    active: bool,
});

// Implement Entity trait for TestUser
impl_entity!(TestUser, "test_users");

#[cfg(test)]
mod typed_query_tests {
    use super::*;
    use crate::config::DatabaseConfig;

    async fn setup_test_db() -> Database {
        let config = DatabaseConfig::sqlite(":memory:");
        Database::connect_with_config(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_typed_query_builder_creation() {
        let db = setup_test_db().await;
        let query = TypedQueryBuilder::<TestUser>::new(&db);
        assert_eq!(query.table_name, "test_users");
    }

    #[tokio::test]
    async fn test_type_safe_field_selection() {
        let db = setup_test_db().await;
        let query = TypedQueryBuilder::<TestUser>::new(&db)
            .select(id)
            .select(name)
            .select(email);

        // Check that the selected fields are tracked
        assert!(query.selected_fields.contains(&"id".to_string()));
        assert!(query.selected_fields.contains(&"name".to_string()));
        assert!(query.selected_fields.contains(&"email".to_string()));
    }

    #[tokio::test]
    async fn test_type_safe_where_conditions() {
        let db = setup_test_db().await;
        let query = TypedQueryBuilder::<TestUser>::new(&db)
            .where_eq(active, true)  // bool parameter
            .where_gt(id, 10i64)     // i64 parameter
            .where_like(name, "test") // String parameter
            .where_in(id, &[1i64, 2, 3]); // Vec<i64> parameter

        // Check that conditions are properly stored
        assert_eq!(query.where_conditions.len(), 4);
    }

    #[tokio::test]
    async fn test_sql_generation_with_type_safe_fields() {
        let db = setup_test_db().await;
        let query = TypedQueryBuilder::<TestUser>::new(&db)
            .select(id)
            .select(name)
            .where_eq(active, true)
            .where_gt(id, 5i64)
            .where_like(name, "John%")
            .where_in(id, &[1i64, 2, 3])
            .order_by(name)
            .order_by_desc(id)
            .limit(10)
            .offset(5);

        let sql = query.build_select_sql();

        // Verify SQL contains expected elements
        assert!(sql.contains("SELECT id, name FROM test_users"));
        assert!(sql.contains("WHERE active = ? AND id > ? AND name LIKE ? AND id IN (?, ?, ?)"));
        assert!(sql.contains("ORDER BY name ASC, id DESC"));
        assert!(sql.contains("LIMIT 10 OFFSET 5"));
    }

    #[tokio::test]
    async fn test_compile_time_field_validation() {
        let db = setup_test_db().await;

        // This should compile - valid fields
        let _query1 = TypedQueryBuilder::<TestUser>::new(&db)
            .where_eq(id, 1i64)
            .where_eq(name, "test".to_string())
            .where_eq(email, "test@example.com".to_string())
            .where_eq(active, true);

        // This would not compile if we tried invalid field types:
        // let _query2 = TestUser::query(&db).where_eq(id, "invalid"); // i64 field with String value
        // let _query3 = TestUser::query(&db).where_eq(nonexistent_field, 123); // nonexistent field
    }

    #[tokio::test]
    async fn test_parameter_binding() {
        let db = setup_test_db().await;
        let query = TypedQueryBuilder::<TestUser>::new(&db)
            .where_eq(active, true)
            .where_gt(id, 10i64)
            .where_like(name, "test%")
            .where_in(id, &[1i64, 2, 3]);

        let params = query.build_parameters();

        // Should have 6 parameters: true, 10, "test%", 1, 2, 3
        assert_eq!(params.len(), 6);
    }

    #[tokio::test]
    async fn test_complex_query_with_all_features() {
        let db = setup_test_db().await;
        let query = TypedQueryBuilder::<TestUser>::new(&db)
            .select(id)
            .select(name)
            .select(email)
            .where_eq(active, true)
            .where_gt(id, 5i64)
            .where_like(name, "John%")
            .where_in(id, &[1i64, 2, 3])
            .order_by(name)
            .order_by_desc(id)
            .limit(10)
            .offset(5);

        let sql = query.build_select_sql();
        let params = query.build_parameters();

        assert!(sql.contains("SELECT id, name, email FROM test_users"));
        assert!(sql.contains("WHERE active = ? AND id > ? AND name LIKE ? AND id IN (?, ?, ?)"));
        assert!(sql.contains("ORDER BY name ASC, id DESC"));
        assert!(sql.contains("LIMIT 10 OFFSET 5"));
        assert_eq!(params.len(), 6);
    }
}