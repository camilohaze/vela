/*!
Entity definitions and metadata for Vela ORM.

This module provides the core traits and derive macros for defining
database entities with type-safe mapping.
*/

use crate::error::{Error, Result, ValidationError, ValidationErrors};
use crate::query::QueryBuilder;
use crate::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for database entities
pub trait Entity: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// Get the table name for this entity
    fn table_name() -> &'static str;

    /// Get the primary key field name
    fn primary_key_field() -> &'static str;

    /// Get the entity metadata
    fn metadata() -> EntityMetadata;

    /// Find entity by ID
    async fn find_by_id(db: &Database, id: i64) -> Result<Option<Self>>
    where
        Self: Sized;

    /// Find all entities
    async fn find_all(db: &Database) -> Result<Vec<Self>>
    where
        Self: Sized;

    /// Save the entity (insert or update)
    async fn save(&self, db: &Database) -> Result<()>
    where
        Self: Sized;

    /// Delete the entity
    async fn delete(&self, db: &Database) -> Result<()>
    where
        Self: Sized;

    /// Create a query builder for this entity
    fn query(db: &Database) -> QueryBuilder<Self>
    where
        Self: Sized;
}

/// Entity metadata containing mapping information
#[derive(Debug, Clone)]
pub struct EntityMetadata {
    /// Table name
    pub table_name: String,
    /// Schema name (optional)
    pub schema: Option<String>,
    /// Primary key field
    pub primary_key: FieldMetadata,
    /// All fields
    pub fields: HashMap<String, FieldMetadata>,
    /// Relations
    pub relations: HashMap<String, RelationMetadata>,
}

impl EntityMetadata {
    /// Create new entity metadata
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            schema: None,
            primary_key: FieldMetadata::default(),
            fields: HashMap::new(),
            relations: HashMap::new(),
        }
    }

    /// Set the schema
    pub fn with_schema(mut self, schema: &str) -> Self {
        self.schema = Some(schema.to_string());
        self
    }

    /// Add a field
    pub fn with_field(mut self, name: &str, field: FieldMetadata) -> Self {
        self.fields.insert(name.to_string(), field);
        self
    }

    /// Set the primary key
    pub fn with_primary_key(mut self, field: FieldMetadata) -> Self {
        self.primary_key = field;
        self
    }

    /// Add a relation
    pub fn with_relation(mut self, name: &str, relation: RelationMetadata) -> Self {
        self.relations.insert(name.to_string(), relation);
        self
    }

    /// Get the fully qualified table name
    pub fn qualified_table_name(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, self.table_name),
            None => self.table_name.clone(),
        }
    }
}

/// Field metadata for entity fields
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    /// Field name in the struct
    pub name: String,
    /// Column name in the database
    pub column_name: String,
    /// SQL type
    pub sql_type: String,
    /// Whether the field is nullable
    pub nullable: bool,
    /// Whether the field is unique
    pub unique: bool,
    /// Whether the field is auto-generated
    pub generated: bool,
    /// Default value expression
    pub default_value: Option<String>,
    /// Field validators
    pub validators: Vec<FieldValidator>,
}

impl Default for FieldMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            column_name: String::new(),
            sql_type: "TEXT".to_string(),
            nullable: true,
            unique: false,
            generated: false,
            default_value: None,
            validators: Vec::new(),
        }
    }
}

impl FieldMetadata {
    /// Create new field metadata
    pub fn new(name: &str, column_name: &str, sql_type: &str) -> Self {
        Self {
            name: name.to_string(),
            column_name: column_name.to_string(),
            sql_type: sql_type.to_string(),
            ..Default::default()
        }
    }

    /// Set nullable
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Set unique
    pub fn unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    /// Set generated
    pub fn generated(mut self, generated: bool) -> Self {
        self.generated = generated;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: &str) -> Self {
        self.default_value = Some(value.to_string());
        self
    }

    /// Add a validator
    pub fn with_validator(mut self, validator: FieldValidator) -> Self {
        self.validators.push(validator);
        self
    }
}

/// Field validator for data validation
#[derive(Debug, Clone)]
pub struct FieldValidator {
    /// Validator type
    pub validator_type: ValidatorType,
    /// Validator parameters
    pub params: HashMap<String, String>,
}

impl FieldValidator {
    /// Create a required validator
    pub fn required() -> Self {
        Self {
            validator_type: ValidatorType::Required,
            params: HashMap::new(),
        }
    }

    /// Create a length validator
    pub fn length(min: Option<usize>, max: Option<usize>) -> Self {
        let mut params = HashMap::new();
        if let Some(min) = min {
            params.insert("min".to_string(), min.to_string());
        }
        if let Some(max) = max {
            params.insert("max".to_string(), max.to_string());
        }

        Self {
            validator_type: ValidatorType::Length,
            params,
        }
    }

    /// Create an email validator
    pub fn email() -> Self {
        Self {
            validator_type: ValidatorType::Email,
            params: HashMap::new(),
        }
    }

    /// Create a range validator
    pub fn range(min: Option<i64>, max: Option<i64>) -> Self {
        let mut params = HashMap::new();
        if let Some(min) = min {
            params.insert("min".to_string(), min.to_string());
        }
        if let Some(max) = max {
            params.insert("max".to_string(), max.to_string());
        }

        Self {
            validator_type: ValidatorType::Range,
            params,
        }
    }
}

/// Types of field validators
#[derive(Debug, Clone)]
pub enum ValidatorType {
    Required,
    Length,
    Email,
    Range,
    Custom(String),
}

/// Relation metadata for entity relationships
#[derive(Debug, Clone)]
pub struct RelationMetadata {
    /// Relation type
    pub relation_type: RelationType,
    /// Target entity type
    pub target_entity: String,
    /// Join column (for many-to-one)
    pub join_column: Option<String>,
    /// Mapped by field (for one-to-many)
    pub mapped_by: Option<String>,
    /// Cascade operations
    pub cascade: Vec<CascadeType>,
    /// Fetch type
    pub fetch: FetchType,
}

impl RelationMetadata {
    /// Create a many-to-one relation
    pub fn many_to_one(target_entity: &str, join_column: &str) -> Self {
        Self {
            relation_type: RelationType::ManyToOne,
            target_entity: target_entity.to_string(),
            join_column: Some(join_column.to_string()),
            mapped_by: None,
            cascade: Vec::new(),
            fetch: FetchType::Lazy,
        }
    }

    /// Create a one-to-many relation
    pub fn one_to_many(target_entity: &str, mapped_by: &str) -> Self {
        Self {
            relation_type: RelationType::OneToMany,
            target_entity: target_entity.to_string(),
            join_column: None,
            mapped_by: Some(mapped_by.to_string()),
            cascade: Vec::new(),
            fetch: FetchType::Lazy,
        }
    }

    /// Create a many-to-many relation
    pub fn many_to_many(target_entity: &str, join_table: &str) -> Self {
        Self {
            relation_type: RelationType::ManyToMany,
            target_entity: target_entity.to_string(),
            join_column: Some(join_table.to_string()),
            mapped_by: None,
            cascade: Vec::new(),
            fetch: FetchType::Lazy,
        }
    }

    /// Add cascade operations
    pub fn with_cascade(mut self, cascade: CascadeType) -> Self {
        self.cascade.push(cascade);
        self
    }

    /// Set fetch type
    pub fn with_fetch(mut self, fetch: FetchType) -> Self {
        self.fetch = fetch;
        self
    }
}

/// Types of entity relations
#[derive(Debug, Clone)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Cascade operations for relations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CascadeType {
    Persist,
    Merge,
    Remove,
    Refresh,
    All,
}

/// Fetch types for relations
#[derive(Debug, Clone)]
pub enum FetchType {
    Lazy,
    Eager,
}

/// Entity manager for CRUD operations
pub struct EntityManager {
    // Implementation would handle entity lifecycle
}

impl EntityManager {
    /// Create a new entity manager
    pub fn new() -> Self {
        Self {}
    }

    /// Validate an entity
    pub fn validate_entity<T: Entity>(&self, entity: &T) -> Result<()> {
        let metadata = T::metadata();
        let mut errors = ValidationErrors::new();

        // Validate fields
        for (field_name, field_meta) in &metadata.fields {
            // Implementation would validate each field
            // This is a placeholder for the actual validation logic
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.into_orm_error())
        }
    }

    /// Generate INSERT SQL for an entity
    pub fn generate_insert_sql<T: Entity>(&self, entity: &T) -> Result<String> {
        let metadata = T::metadata();
        let table_name = metadata.qualified_table_name();

        let mut columns = Vec::new();
        let mut placeholders = Vec::new();

        for (field_name, field_meta) in &metadata.fields {
            if !field_meta.generated {
                columns.push(field_meta.column_name.clone());
                placeholders.push("?".to_string());
            }
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        Ok(sql)
    }

    /// Generate UPDATE SQL for an entity
    pub fn generate_update_sql<T: Entity>(&self, entity: &T) -> Result<String> {
        let metadata = T::metadata();
        let table_name = metadata.qualified_table_name();

        let mut sets = Vec::new();

        for (field_name, field_meta) in &metadata.fields {
            if field_name != &metadata.primary_key.name && !field_meta.generated {
                sets.push(format!("{} = ?", field_meta.column_name));
            }
        }

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ?",
            table_name,
            sets.join(", "),
            metadata.primary_key.column_name
        );

        Ok(sql)
    }

    /// Generate DELETE SQL for an entity
    pub fn generate_delete_sql<T: Entity>(&self, entity: &T) -> Result<String> {
        let metadata = T::metadata();
        let table_name = metadata.qualified_table_name();

        let sql = format!(
            "DELETE FROM {} WHERE {} = ?",
            table_name,
            metadata.primary_key.column_name
        );

        Ok(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_metadata() {
        let mut metadata = EntityMetadata::new("users")
            .with_schema("public")
            .with_field("name", FieldMetadata::new("name", "name", "VARCHAR(255)")
                .nullable(false)
                .with_validator(FieldValidator::required())
                .with_validator(FieldValidator::length(Some(2), Some(100))));

        assert_eq!(metadata.table_name, "users");
        assert_eq!(metadata.schema, Some("public".to_string()));
        assert_eq!(metadata.qualified_table_name(), "public.users");
        assert_eq!(metadata.fields.len(), 1);
    }

    #[test]
    fn test_field_validators() {
        let validator = FieldValidator::email();
        assert!(matches!(validator.validator_type, ValidatorType::Email));

        let validator = FieldValidator::length(Some(5), Some(50));
        assert!(matches!(validator.validator_type, ValidatorType::Length));
        assert_eq!(validator.params.get("min"), Some(&"5".to_string()));
        assert_eq!(validator.params.get("max"), Some(&"50".to_string()));
    }

    #[test]
    fn test_relation_metadata() {
        let relation = RelationMetadata::many_to_one("Department", "department_id")
            .with_cascade(CascadeType::Persist)
            .with_fetch(FetchType::Eager);

        assert!(matches!(relation.relation_type, RelationType::ManyToOne));
        assert_eq!(relation.target_entity, "Department");
        assert_eq!(relation.join_column, Some("department_id".to_string()));
        assert_eq!(relation.cascade.len(), 1);
        assert!(matches!(relation.fetch, FetchType::Eager));
    }
}