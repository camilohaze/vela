/*!
Type-safe query builder for Vela ORM.

This module provides compile-time type safety for database queries,
including field validation, type checking, and IDE autocompletion.
*/

use crate::connection::{Database, QueryResult, ToSql, Value};
use crate::entity::Entity;
use crate::error::{Error, Result};
use std::marker::PhantomData;

/// Marker trait for entity fields with type safety
pub trait Field<T> {
    /// Field name as it appears in the database
    const NAME: &'static str;
    /// The entity type this field belongs to
    type Entity: Entity;
}

/// Type-safe field marker for an entity field
#[derive(Debug, Clone)]
pub struct FieldMarker<F: Field<T>, T> {
    _phantom: PhantomData<(F, T)>,
}

impl<F: Field<T>, T> FieldMarker<F, T> {
    /// Create a new field marker
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Type-safe query builder with compile-time field validation
pub struct TypedQueryBuilder<E: Entity> {
    db: Database,
    table_name: String,
    selected_fields: Vec<String>,
    conditions: Vec<TypedWhereCondition>,
    join_clauses: Vec<TypedJoin>,
    order_by: Vec<TypedOrderBy>,
    group_by: Vec<String>,
    having_conditions: Vec<TypedWhereCondition>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phantom: PhantomData<E>,
}

impl<E: Entity> TypedQueryBuilder<E> {
    /// Create a new type-safe query builder
    pub fn new(db: &Database) -> Self {
        Self {
            db: db.clone(),
            table_name: E::table_name().to_string(),
            selected_fields: vec!["*".to_string()],
            conditions: Vec::new(),
            join_clauses: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having_conditions: Vec::new(),
            limit: None,
            offset: None,
            _phantom: PhantomData,
        }
    }

    /// Select all fields (*)
    pub fn select_all(mut self) -> Self {
        self.selected_fields = vec!["*".to_string()];
        self
    }

    /// Select specific fields (type-safe)
    pub fn select<F, T>(mut self, _field: F) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
    {
        if self.selected_fields.len() == 1 && self.selected_fields[0] == "*" {
            self.selected_fields.clear();
        }
        self.selected_fields.push(F::NAME.to_string());
        self
    }

    /// Add WHERE condition with equality (type-safe)
    pub fn where_eq<F, T>(mut self, _field: F, value: T) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::Equal,
            value: value.to_sql(),
        });
        self
    }

    /// Add WHERE condition with inequality (type-safe)
    pub fn where_ne<F, T>(mut self, _field: F, value: T) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::NotEqual,
            value: value.to_sql(),
        });
        self
    }

    /// Add WHERE condition with greater than (type-safe)
    pub fn where_gt<F, T>(mut self, _field: F, value: T) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::GreaterThan,
            value: value.to_sql(),
        });
        self
    }

    /// Add WHERE condition with greater than or equal (type-safe)
    pub fn where_gte<F, T>(mut self, _field: F, value: T) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::GreaterThanOrEqual,
            value: value.to_sql(),
        });
        self
    }

    /// Add WHERE condition with less than (type-safe)
    pub fn where_lt<F, T>(mut self, _field: F, value: T) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::LessThan,
            value: value.to_sql(),
        });
        self
    }

    /// Add WHERE condition with less than or equal (type-safe)
    pub fn where_lte<F, T>(mut self, _field: F, value: T) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::LessThanOrEqual,
            value: value.to_sql(),
        });
        self
    }

    /// Add WHERE condition with LIKE (type-safe)
    pub fn where_like<F>(mut self, _field: F, pattern: &str) -> Self
    where
        F: Field<String>,
        F::Entity: Entity,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::Like,
            value: Value::String(pattern.to_string()),
        });
        self
    }

    /// Add WHERE condition with IN (type-safe)
    pub fn where_in<F, T>(mut self, _field: F, values: &[T]) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
        T: ToSql + Clone,
    {
        let sql_values: Vec<Value> = values.iter().map(|v| v.to_sql()).collect();
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::In,
            value: Value::Json(serde_json::to_value(sql_values).unwrap()),
        });
        self
    }

    /// Add WHERE condition for NULL check (type-safe)
    pub fn where_null<F, T>(mut self, _field: F) -> Self
    where
        F: Field<Option<T>>,
        F::Entity: Entity,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::IsNull,
            value: Value::Null,
        });
        self
    }

    /// Add WHERE condition for NOT NULL check (type-safe)
    pub fn where_not_null<F, T>(mut self, _field: F) -> Self
    where
        F: Field<Option<T>>,
        F::Entity: Entity,
    {
        self.conditions.push(TypedWhereCondition {
            field: F::NAME.to_string(),
            operator: TypedOperator::IsNotNull,
            value: Value::Null,
        });
        self
    }

    /// Add ORDER BY clause (type-safe)
    pub fn order_by<F, T>(mut self, _field: F) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
    {
        self.order_by.push(TypedOrderBy {
            field: F::NAME.to_string(),
            direction: OrderDirection::Asc,
        });
        self
    }

    /// Add ORDER BY DESC clause (type-safe)
    pub fn order_by_desc<F, T>(mut self, _field: F) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
    {
        self.order_by.push(TypedOrderBy {
            field: F::NAME.to_string(),
            direction: OrderDirection::Desc,
        });
        self
    }

    /// Add GROUP BY clause (type-safe)
    pub fn group_by<F, T>(mut self, _field: F) -> Self
    where
        F: Field<T>,
        F::Entity: Entity,
    {
        self.group_by.push(F::NAME.to_string());
        self
    }

    /// Add LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add OFFSET clause
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Add INNER JOIN clause
    pub fn inner_join<U: Entity>(mut self, on_condition: &str) -> Self {
        self.join_clauses.push(TypedJoin {
            join_type: JoinType::Inner,
            table: U::table_name().to_string(),
            on: on_condition.to_string(),
        });
        self
    }

    /// Add LEFT JOIN clause
    pub fn left_join<U: Entity>(mut self, on_condition: &str) -> Self {
        self.join_clauses.push(TypedJoin {
            join_type: JoinType::Left,
            table: U::table_name().to_string(),
            on: on_condition.to_string(),
        });
        self
    }

    /// Add RIGHT JOIN clause
    pub fn right_join<U: Entity>(mut self, on_condition: &str) -> Self {
        self.join_clauses.push(TypedJoin {
            join_type: JoinType::Right,
            table: U::table_name().to_string(),
            on: on_condition.to_string(),
        });
        self
    }

    /// Add FULL OUTER JOIN clause
    pub fn full_join<U: Entity>(mut self, on_condition: &str) -> Self {
        self.join_clauses.push(TypedJoin {
            join_type: JoinType::Full,
            table: U::table_name().to_string(),
            on: on_condition.to_string(),
        });
        self
    }

    /// Execute the query and return multiple results
    pub async fn find_many(self) -> Result<Vec<E>> {
        let sql = self.build_select_sql();
        let params = self.build_params();

        let result = self.db.query(&sql, params).await?;
        self.result_to_entities(result)
    }

    /// Execute the query and return a single result
    pub async fn find_one(self) -> Result<Option<E>> {
        let result = self.limit(1).find_many().await?;
        Ok(result.into_iter().next())
    }

    /// Execute the query and return the first result
    pub async fn find_first(self) -> Result<Option<E>> {
        self.find_one().await
    }

    /// Execute a COUNT query
    pub async fn count(self) -> Result<i64> {
        let mut count_query = self;
        count_query.selected_fields = vec!["COUNT(*) as count".to_string()];

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
    pub fn build_select_sql(&self) -> String {
        let table_name = E::table_name();
        let mut sql = format!("SELECT {} FROM {}", self.selected_fields.join(", "), table_name);

        // JOIN clauses
        for join in &self.join_clauses {
            sql.push_str(&format!(" {} JOIN {} ON {}", join.join_type.as_sql(), join.table, join.on));
        }

        // WHERE conditions
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self.conditions.iter()
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
        for condition in &self.conditions {
            if let TypedOperator::In = condition.operator {
                // For IN clauses, expand the array
                if let Value::Json(json) = &condition.value {
                    if let Ok(values) = serde_json::from_value::<Vec<Value>>(json.clone()) {
                        for value in values {
                            params.push(Box::new(SqlParam(value)) as Box<dyn ToSql>);
                        }
                    }
                }
            } else if !matches!(condition.operator, TypedOperator::IsNull | TypedOperator::IsNotNull) {
                params.push(Box::new(SqlParam(condition.value.clone())) as Box<dyn ToSql>);
            }
        }

        // Add HAVING condition parameters
        for condition in &self.having_conditions {
            if !matches!(condition.operator, TypedOperator::IsNull | TypedOperator::IsNotNull) {
                params.push(Box::new(SqlParam(condition.value.clone())) as Box<dyn ToSql>);
            }
        }

        params
    }

    /// Convert query result to entities
    fn result_to_entities(&self, result: QueryResult) -> Result<Vec<E>> {
        let mut entities = Vec::new();

        for row in result.rows {
            // This would use the entity metadata to map columns to fields
            // For now, we'll use serde deserialization as a placeholder
            let json_value = serde_json::json!({}); // Placeholder
            let entity: E = serde_json::from_value(json_value)
                .map_err(|e| Error::serialization(format!("Failed to deserialize entity: {}", e)))?;
            entities.push(entity);
        }

        Ok(entities)
    }
}

/// Type-safe WHERE condition
#[derive(Debug, Clone)]
struct TypedWhereCondition {
    field: String,
    operator: TypedOperator,
    value: Value,
}

impl TypedWhereCondition {
    fn to_sql(&self) -> String {
        match self.operator {
            TypedOperator::Equal => format!("{} = ?", self.field),
            TypedOperator::NotEqual => format!("{} != ?", self.field),
            TypedOperator::GreaterThan => format!("{} > ?", self.field),
            TypedOperator::GreaterThanOrEqual => format!("{} >= ?", self.field),
            TypedOperator::LessThan => format!("{} < ?", self.field),
            TypedOperator::LessThanOrEqual => format!("{} <= ?", self.field),
            TypedOperator::Like => format!("{} LIKE ?", self.field),
            TypedOperator::In => format!("{} IN ({})", self.field, self.build_in_placeholders()),
            TypedOperator::IsNull => format!("{} IS NULL", self.field),
            TypedOperator::IsNotNull => format!("{} IS NOT NULL", self.field),
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

/// Type-safe WHERE operators
#[derive(Debug, Clone)]
enum TypedOperator {
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

/// Type-safe JOIN clause
#[derive(Debug, Clone)]
struct TypedJoin {
    join_type: JoinType,
    table: String,
    on: String,
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

/// Type-safe ORDER BY clause
#[derive(Debug, Clone)]
struct TypedOrderBy {
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

/// Macro to generate type-safe field markers for an entity
#[macro_export]
macro_rules! entity_fields {
    ($entity:ty { $($field:ident: $type:ty),* $(,)? }) => {
        $(
            #[derive(Debug, Clone)]
            pub struct $field;

            impl Field<$type> for $field {
                const NAME: &'static str = stringify!($field);
                type Entity = $entity;
            }
        )*
    };
}

/// Macro to implement Entity trait with type-safe query builder
#[macro_export]
macro_rules! impl_entity {
    ($entity:ty, $table_name:expr) => {
        impl Entity for $entity {
            fn table_name() -> &'static str {
                $table_name
            }

            fn primary_key_field() -> &'static str {
                "id"
            }

            fn metadata() -> crate::entity::EntityMetadata {
                crate::entity::EntityMetadata::new($table_name)
            }

            async fn find_by_id(db: &Database, id: i64) -> Result<Option<Self>> {
                Self::query(db).where_eq(crate::typed_query::id, id).find_one().await
            }

            async fn find_all(db: &Database) -> Result<Vec<Self>> {
                Self::query(db).find_many().await
            }

            async fn save(&self, _db: &Database) -> Result<()> {
                // TODO: Implement save logic
                Ok(())
            }

            async fn delete(&self, _db: &Database) -> Result<()> {
                // TODO: Implement delete logic
                Ok(())
            }

            fn query(db: &Database) -> TypedQueryBuilder<Self> {
                TypedQueryBuilder::new(db.clone())
            }
        }
    };
}

