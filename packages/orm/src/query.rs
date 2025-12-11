/*!
Type-safe query builder for Vela ORM.

This module provides a fluent API for building database queries
with compile-time type safety and IDE autocompletion.
*/

use crate::connection::{Database, QueryResult, ToSql, Value};
use crate::entity::Entity;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Query builder for type-safe database queries
pub struct QueryBuilder<T: Entity> {
    db: Database,
    table_name: String,
    select_fields: Vec<String>,
    where_conditions: Vec<WhereCondition>,
    join_clauses: Vec<JoinClause>,
    order_by: Vec<OrderBy>,
    group_by: Vec<String>,
    having_conditions: Vec<WhereCondition>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phantom: PhantomData<T>,
}

impl<T: Entity> QueryBuilder<T> {
    /// Create a new query builder
    pub fn new(db: Database) -> Self {
        Self {
            db,
            table_name: T::table_name().to_string(),
            select_fields: vec!["*".to_string()],
            where_conditions: Vec::new(),
            join_clauses: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having_conditions: Vec::new(),
            limit: None,
            offset: None,
            _phantom: PhantomData,
        }
    }

    /// Select specific fields
    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a WHERE condition with equality
    pub fn where_eq<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::Equal,
            value: value.to_sql(),
        });
        self
    }

    /// Add a WHERE condition with inequality
    pub fn where_ne<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::NotEqual,
            value: value.to_sql(),
        });
        self
    }

    /// Add a WHERE condition with greater than
    pub fn where_gt<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::GreaterThan,
            value: value.to_sql(),
        });
        self
    }

    /// Add a WHERE condition with greater than or equal
    pub fn where_gte<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::GreaterThanOrEqual,
            value: value.to_sql(),
        });
        self
    }

    /// Add a WHERE condition with less than
    pub fn where_lt<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::LessThan,
            value: value.to_sql(),
        });
        self
    }

    /// Add a WHERE condition with less than or equal
    pub fn where_lte<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::LessThanOrEqual,
            value: value.to_sql(),
        });
        self
    }

    /// Add a WHERE condition with LIKE
    pub fn where_like(mut self, field: &str, pattern: &str) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::Like,
            value: Value::String(pattern.to_string()),
        });
        self
    }

    /// Add a WHERE condition with IN
    pub fn where_in<S: ToSql>(mut self, field: &str, values: &[S]) -> Self {
        let sql_values: Vec<Value> = values.iter().map(|v| v.to_sql()).collect();
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::In,
            value: Value::Json(serde_json::to_value(sql_values).unwrap()),
        });
        self
    }

    /// Add a WHERE condition with IS NULL
    pub fn where_null(mut self, field: &str) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::IsNull,
            value: Value::Null,
        });
        self
    }

    /// Add a WHERE condition with IS NOT NULL
    pub fn where_not_null(mut self, field: &str) -> Self {
        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::IsNotNull,
            value: Value::Null,
        });
        self
    }

    /// Add an INNER JOIN
    pub fn join(mut self, table: &str, on: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// Add a LEFT JOIN
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// Add a RIGHT JOIN
    pub fn right_join(mut self, table: &str, on: &str) -> Self {
        self.join_clauses.push(JoinClause {
            join_type: JoinType::Right,
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by.push(OrderBy {
            field: field.to_string(),
            direction: OrderDirection::Asc,
        });
        self
    }

    /// Add an ORDER BY DESC clause
    pub fn order_by_desc(mut self, field: &str) -> Self {
        self.order_by.push(OrderBy {
            field: field.to_string(),
            direction: OrderDirection::Desc,
        });
        self
    }

    /// Add a GROUP BY clause
    pub fn group_by(mut self, fields: &[&str]) -> Self {
        self.group_by = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a HAVING condition
    pub fn having_eq<S: ToSql>(mut self, field: &str, value: S) -> Self {
        self.having_conditions.push(WhereCondition {
            field: field.to_string(),
            operator: WhereOperator::Equal,
            value: value.to_sql(),
        });
        self
    }

    /// Add a LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add an OFFSET clause
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Execute the query and return multiple results
    pub async fn find_many(self) -> Result<Vec<T>> {
        let sql = self.build_select_sql();
        let params = self.build_params();

        let result = self.db.query(&sql, params).await?;
        let entities = self.result_to_entities(result)?;

        Ok(entities)
    }

    /// Execute the query and return a single result
    pub async fn find_one(self) -> Result<Option<T>> {
        let result = self.limit(1).find_many().await?;
        Ok(result.into_iter().next())
    }

    /// Execute the query and return the first result
    pub async fn find_first(self) -> Result<Option<T>> {
        self.find_one().await
    }

    /// Execute a COUNT query
    pub async fn count(self) -> Result<i64> {
        let mut count_query = self;
        count_query.select_fields = vec!["COUNT(*) as count".to_string()];

        let sql = count_query.build_select_sql();
        let params = count_query.build_params();

        let result = count_query.db.query(&sql, params).await?;

        if let Some(row) = result.rows.first() {
            if let Some(Value::I64(count)) = row.get(0) {
                return Ok(*count);
            }
        }

        Err(Error::query("Failed to get count result"))
    }

    /// Execute an EXISTS query
    pub async fn exists(self) -> Result<bool> {
        let count = self.count().await?;
        Ok(count > 0)
    }

    /// Build the SELECT SQL statement
    fn build_select_sql(&self) -> String {
        let mut sql = format!("SELECT {} FROM {}", self.select_fields.join(", "), self.table_name);

        // JOIN clauses
        for join in &self.join_clauses {
            sql.push_str(&format!(" {} JOIN {} ON {}", join.join_type.as_sql(), join.table, join.on));
        }

        // WHERE conditions
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self.where_conditions.iter()
                .map(|c| c.to_sql())
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING conditions
        if !self.having_conditions.is_empty() {
            sql.push_str(" HAVING ");
            let conditions: Vec<String> = self.having_conditions.iter()
                .map(|c| c.to_sql())
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self.order_by.iter()
                .map(|o| format!("{} {}", o.field, o.direction.as_sql()))
                .collect();
            sql.push_str(&orders.join(", "));
        }

        // LIMIT and OFFSET
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    /// Build the parameter list for the query
    fn build_params(&self) -> Vec<Box<dyn ToSql>> {
        let mut params = Vec::new();

        // Add WHERE condition parameters
        for condition in &self.where_conditions {
            if let WhereOperator::In = condition.operator {
                // For IN clauses, expand the array
                if let Value::Json(json) = &condition.value {
                    if let Ok(values) = serde_json::from_value::<Vec<Value>>(json.clone()) {
                        for value in values {
                            params.push(Box::new(SqlParam(value)) as Box<dyn ToSql>);
                        }
                    }
                }
            } else if !matches!(condition.operator, WhereOperator::IsNull | WhereOperator::IsNotNull) {
                params.push(Box::new(SqlParam(condition.value.clone())) as Box<dyn ToSql>);
            }
        }

        // Add HAVING condition parameters
        for condition in &self.having_conditions {
            if !matches!(condition.operator, WhereOperator::IsNull | WhereOperator::IsNotNull) {
                params.push(Box::new(SqlParam(condition.value.clone())) as Box<dyn ToSql>);
            }
        }

        params
    }

    /// Convert query result to entities
    fn result_to_entities(&self, result: QueryResult) -> Result<Vec<T>> {
        let mut entities = Vec::new();

        for row in result.rows {
            // This would use the entity metadata to map columns to fields
            // For now, we'll use serde deserialization as a placeholder
            let json_value = serde_json::json!({}); // Placeholder
            let entity: T = serde_json::from_value(json_value)
                .map_err(|e| Error::serialization(format!("Failed to deserialize entity: {}", e)))?;
            entities.push(entity);
        }

        Ok(entities)
    }
}

/// WHERE condition for queries
#[derive(Debug, Clone)]
struct WhereCondition {
    field: String,
    operator: WhereOperator,
    value: Value,
}

impl WhereCondition {
    fn to_sql(&self) -> String {
        match self.operator {
            WhereOperator::Equal => format!("{} = ?", self.field),
            WhereOperator::NotEqual => format!("{} != ?", self.field),
            WhereOperator::GreaterThan => format!("{} > ?", self.field),
            WhereOperator::GreaterThanOrEqual => format!("{} >= ?", self.field),
            WhereOperator::LessThan => format!("{} < ?", self.field),
            WhereOperator::LessThanOrEqual => format!("{} <= ?", self.field),
            WhereOperator::Like => format!("{} LIKE ?", self.field),
            WhereOperator::In => format!("{} IN ({})", self.field, self.build_in_placeholders()),
            WhereOperator::IsNull => format!("{} IS NULL", self.field),
            WhereOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
        }
    }

    fn build_in_placeholders(&self) -> String {
        if let Value::Json(json) = &self.value {
            if let Ok(values) = serde_json::from_value::<Vec<Value>>(json.clone()) {
                return vec!["?"; values.len()].join(", ");
            }
        }
        "?".to_string()
    }
}

/// WHERE operators
#[derive(Debug, Clone)]
enum WhereOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    In,
    IsNull,
    IsNotNull,
}

/// JOIN clause for queries
#[derive(Debug, Clone)]
struct JoinClause {
    join_type: JoinType,
    table: String,
    on: String,
}

impl JoinClause {
    fn as_sql(&self) -> String {
        format!("{} JOIN {} ON {}", self.join_type.as_sql(), self.table, self.on)
    }
}

/// JOIN types
#[derive(Debug, Clone)]
enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl JoinType {
    fn as_sql(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER",
            JoinType::Left => "LEFT",
            JoinType::Right => "RIGHT",
            JoinType::Full => "FULL",
        }
    }
}

/// ORDER BY clause
#[derive(Debug, Clone)]
struct OrderBy {
    field: String,
    direction: OrderDirection,
}

/// ORDER BY directions
#[derive(Debug, Clone)]
enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    fn as_sql(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// Wrapper for SQL parameters
struct SqlParam(Value);

impl ToSql for SqlParam {
    fn to_sql(&self) -> Value {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    // Mock entity for testing
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestEntity;

    impl Entity for TestEntity {
        fn table_name() -> &'static str { "test_entities" }
        fn primary_key_field() -> &'static str { "id" }
        fn metadata() -> crate::entity::EntityMetadata {
            crate::entity::EntityMetadata::new("test_entities")
        }

        async fn find_by_id(_db: &Database, _id: i64) -> Result<Option<Self>> { Ok(None) }
        async fn find_all(_db: &Database) -> Result<Vec<Self>> { Ok(vec![]) }
        async fn save(&self, _db: &Database) -> Result<()> { Ok(()) }
        async fn delete(&self, _db: &Database) -> Result<()> { Ok(()) }
        fn query(db: &Database) -> QueryBuilder<Self> { QueryBuilder::new(db.clone()) }
    }

    #[tokio::test]
    async fn test_query_builder_basic() {
        let config = DatabaseConfig::sqlite(":memory:");
        let db = Database::connect_with_config(config).await.unwrap();

        let query = TestEntity::query(&db)
            .where_eq("name", "test")
            .where_gt("age", 18)
            .order_by("name")
            .limit(10);

        let sql = query.build_select_sql();
        assert!(sql.contains("SELECT * FROM test_entities"));
        assert!(sql.contains("WHERE name = ? AND age > ?"));
        assert!(sql.contains("ORDER BY name ASC"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[tokio::test]
    async fn test_where_conditions() {
        let config = DatabaseConfig::sqlite(":memory:");
        let db = Database::connect_with_config(config).await.unwrap();

        let query = TestEntity::query(&db)
            .where_eq("status", "active")
            .where_like("name", "John%")
            .where_in("id", &[1i64, 2, 3])
            .where_null("deleted_at");

        let sql = query.build_select_sql();
        assert!(sql.contains("status = ?"));
        assert!(sql.contains("name LIKE ?"));
        assert!(sql.contains("id IN (?, ?, ?)"));
        assert!(sql.contains("deleted_at IS NULL"));
    }

    #[tokio::test]
    async fn test_joins_and_grouping() {
        let config = DatabaseConfig::sqlite(":memory:");
        let db = Database::connect_with_config(config).await.unwrap();

        let query = TestEntity::query(&db)
            .join("departments", "test_entities.dept_id = departments.id")
            .left_join("permissions", "test_entities.id = permissions.user_id")
            .group_by(&["department_id"])
            .having_eq("COUNT(*)", 5)
            .order_by_desc("created_at");

        let sql = query.build_select_sql();
        assert!(sql.contains("INNER JOIN departments"));
        assert!(sql.contains("LEFT JOIN permissions"));
        assert!(sql.contains("GROUP BY department_id"));
        assert!(sql.contains("HAVING COUNT(*) = ?"));
        assert!(sql.contains("ORDER BY created_at DESC"));
    }
}