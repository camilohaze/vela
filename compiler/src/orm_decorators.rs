//! ORM decorators for Vela compiler
//!
//! This module handles parsing and code generation for ORM decorators
//! like @entity, @id, @column, @oneToMany, @manyToOne, etc.

use crate::ast::{Decorator, Expression, Literal, Statement, TypeAnnotation, BinaryExpression};
use crate::error::{CompileError, ParseError, SourceLocation};
use std::collections::HashMap;

/// Enum representing all ORM decorators
#[derive(Debug, Clone)]
pub enum OrmDecorator {
    Entity(EntityDecorator),
    Id(IdDecorator),
    Column(ColumnDecorator),
    OneToMany(OneToManyDecorator),
    ManyToOne(ManyToOneDecorator),
    ManyToMany(ManyToManyDecorator),
}

/// Entity decorator configuration
#[derive(Debug, Clone)]
pub struct EntityDecorator {
    pub table: Option<String>,
    pub schema: Option<String>,
}

/// ID decorator configuration
#[derive(Debug, Clone)]
pub struct IdDecorator {
    pub generated: bool,
}

/// Column decorator configuration
#[derive(Debug, Clone)]
pub struct ColumnDecorator {
    pub name: Option<String>,
    pub nullable: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub generated: bool,
    pub default: Option<String>,
}

/// One-to-many relation decorator configuration
#[derive(Debug, Clone)]
pub struct OneToManyDecorator {
    pub entity: String,
    pub mapped_by: String,
    pub cascade: Vec<String>,
    pub fetch: String,
}

/// Many-to-one relation decorator configuration
#[derive(Debug, Clone)]
pub struct ManyToOneDecorator {
    pub entity: String,
    pub join_column: String,
    pub cascade: Vec<String>,
    pub fetch: String,
}

/// Many-to-many relation decorator configuration
#[derive(Debug, Clone)]
pub struct ManyToManyDecorator {
    pub entity: String,
    pub join_table: String,
    pub join_column: String,
    pub inverse_join_column: String,
    pub cascade: Vec<String>,
    pub fetch: String,
}

/// Parse ORM decorators from a list of decorators
pub fn parse_orm_decorators(
    decorators: &[Decorator],
) -> Result<Vec<OrmDecorator>, CompileError> {
    let mut orm_decorators = Vec::new();

    for decorator in decorators {
        match decorator.name.as_str() {
            "entity" => {
                let config = parse_entity_decorator(decorator)?;
                orm_decorators.push(OrmDecorator::Entity(config));
            }
            "id" => {
                let config = parse_id_decorator(decorator)?;
                orm_decorators.push(OrmDecorator::Id(config));
            }
            "column" => {
                let config = parse_column_decorator(decorator)?;
                orm_decorators.push(OrmDecorator::Column(config));
            }
            "oneToMany" => {
                let config = parse_one_to_many_decorator(decorator)?;
                orm_decorators.push(OrmDecorator::OneToMany(config));
            }
            "manyToOne" => {
                let config = parse_many_to_one_decorator(decorator)?;
                orm_decorators.push(OrmDecorator::ManyToOne(config));
            }
            "manyToMany" => {
                let config = parse_many_to_many_decorator(decorator)?;
                orm_decorators.push(OrmDecorator::ManyToMany(config));
            }
            _ => {
                // Not an ORM decorator, skip
                continue;
            }
        }
    }

    Ok(orm_decorators)
}

/// Parse entity decorator arguments
pub fn parse_entity_decorator(
    decorator: &Decorator,
) -> Result<EntityDecorator, CompileError> {
    let mut table = None;
    let mut schema = None;

    for arg in &decorator.arguments {
        if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
            if operator == "=" {
                if let Expression::Identifier(key) = left.as_ref() {
                    match key.name.as_str() {
                        "table" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    table = Some(value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for table".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string());
                                }
                            }
                        }
                        "schema" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    schema = Some(value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for schema".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string());
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::Parse(ParseError {
                                message: format!("Unknown entity decorator parameter: {}", key.name),
                                location: SourceLocation::new(key.node.range.start.line, key.node.range.start.column, 0),
                                expected: vec!["table".to_string(), "schema".to_string()],
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(EntityDecorator { table, schema })
}

/// Parse ID decorator arguments
pub fn parse_id_decorator(
    decorator: &Decorator,
) -> Result<IdDecorator, CompileError> {
    let mut generated = false;

    for arg in &decorator.arguments {
        if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
            if operator == "=" {
                if let Expression::Identifier(key) = left.as_ref() {
                    match key.name.as_str() {
                        "generated" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "boolean" {
                                    generated = value.value.as_bool()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid boolean value for generated".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["boolean".to_string()],
                                        }))?;
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::Parse(ParseError {
                                message: format!("Unknown id decorator parameter: {}", key.name),
                                location: SourceLocation::new(key.node.range.start.line, key.node.range.start.column, 0),
                                expected: vec!["generated".to_string()],
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(IdDecorator { generated })
}

/// Parse column decorator arguments
pub fn parse_column_decorator(
    decorator: &Decorator,
) -> Result<ColumnDecorator, CompileError> {
    let mut name = None;
    let mut nullable = false;
    let mut unique = false;
    let mut primary_key = false;
    let mut generated = false;
    let mut default = None;

    for arg in &decorator.arguments {
        if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
            if operator == "=" {
                if let Expression::Identifier(key) = left.as_ref() {
                    match key.name.as_str() {
                        "name" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    name = Some(value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for name".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string());
                                }
                            }
                        }
                        "nullable" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "boolean" {
                                    nullable = value.value.as_bool()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid boolean value for nullable".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["boolean".to_string()],
                                        }))?;
                                }
                            }
                        }
                        "unique" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "boolean" {
                                    unique = value.value.as_bool()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid boolean value for unique".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["boolean".to_string()],
                                        }))?;
                                }
                            }
                        }
                        "primary_key" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "boolean" {
                                    primary_key = value.value.as_bool()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid boolean value for primary_key".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["boolean".to_string()],
                                        }))?;
                                }
                            }
                        }
                        "generated" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "boolean" {
                                    generated = value.value.as_bool()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid boolean value for generated".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["boolean".to_string()],
                                        }))?;
                                }
                            }
                        }
                        "default" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    default = Some(value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for default".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string());
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::Parse(ParseError {
                                message: format!("Unknown column decorator parameter: {}", key.name),
                                location: SourceLocation::new(key.node.range.start.line, key.node.range.start.column, 0),
                                expected: vec!["name".to_string(), "nullable".to_string(), "unique".to_string(), "primary_key".to_string(), "generated".to_string(), "default".to_string()],
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(ColumnDecorator {
        name,
        nullable,
        unique,
        primary_key,
        generated,
        default,
    })
}

/// Parse one-to-many decorator arguments
pub fn parse_one_to_many_decorator(
    decorator: &Decorator,
) -> Result<OneToManyDecorator, CompileError> {
    let mut entity = String::new();
    let mut mapped_by = String::new();
    let mut cascade = Vec::new();
    let mut fetch = "lazy".to_string();

    for arg in &decorator.arguments {
        if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
            if operator == "=" {
                if let Expression::Identifier(key) = left.as_ref() {
                    match key.name.as_str() {
                        "entity" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    entity = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for entity".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        "mapped_by" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    mapped_by = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for mapped_by".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                            }
                        "cascade" => {
                            // Handle array of strings
                            if let Expression::ArrayLiteral(array) = right.as_ref() {
                                for item in &array.elements {
                                    if let Expression::Literal(value) = item {
                                        if value.kind == "string" {
                                            cascade.push(value.value.as_str()
                                                .ok_or_else(|| CompileError::Parse(ParseError {
                                                    message: "Invalid string value in cascade array".to_string(),
                                                    location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                                    expected: vec!["string".to_string()],
                                                }))?
                                                .to_string());
                                        }
                                    }
                                }
                            }
                        }
                        "fetch" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    fetch = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for fetch".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::Parse(ParseError {
                                message: format!("Unknown oneToMany decorator parameter: {}", key.name),
                                location: SourceLocation::new(key.node.range.start.line, key.node.range.start.column, 0),
                                expected: vec!["entity".to_string(), "mapped_by".to_string(), "cascade".to_string(), "fetch".to_string()],
                            }));
                        }
                    }
                }
            }
        }
    }

    if entity.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "oneToMany decorator requires entity parameter".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["entity".to_string()],
        }));
    }

    if mapped_by.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "oneToMany decorator requires mapped_by parameter".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["mapped_by".to_string()],
        }));
    }

    Ok(OneToManyDecorator {
        entity,
        mapped_by,
        cascade,
        fetch,
    })
}

/// Parse many-to-one decorator arguments
pub fn parse_many_to_one_decorator(
    decorator: &Decorator,
) -> Result<ManyToOneDecorator, CompileError> {
    let mut entity = String::new();
    let mut join_column = String::new();
    let mut cascade = Vec::new();
    let mut fetch = "lazy".to_string();

    for arg in &decorator.arguments {
        if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
            if operator == "=" {
                if let Expression::Identifier(key) = left.as_ref() {
                    match key.name.as_str() {
                        "entity" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    entity = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for entity".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        "join_column" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    join_column = Some(value.value.as_str().unwrap().to_string())
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for join_column".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        "cascade" => {
                            // Handle array of strings
                            if let Expression::ArrayLiteral(array) = right.as_ref() {
                                for item in &array.elements {
                                    if let Expression::Literal(value) = item {
                                        if value.kind == "string" {
                                            cascade.push(value.value.as_str()
                                                .ok_or_else(|| CompileError::Parse(ParseError {
                                                    message: "Invalid string value in cascade array".to_string(),
                                                    location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                                    expected: vec!["string".to_string()],
                                                }))?
                                                .to_string());
                                        }
                                    }
                                }
                            }
                        }
                        "fetch" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    fetch = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for fetch".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::Parse(ParseError {
                                message: format!("Unknown manyToOne decorator parameter: {}", key.name),
                                location: SourceLocation::new(key.node.range.start.line, key.node.range.start.column, 0),
                                expected: vec!["entity".to_string(), "join_column".to_string(), "cascade".to_string(), "fetch".to_string()],
                            }));
                        }
                    }
                }
            }
        }
    }

    if entity.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "manyToOne decorator requires entity parameter".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["entity".to_string()],
        }));
    }

    if join_column.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "manyToOne decorator requires join_column parameter".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["join_column".to_string()],
        }));
    }

    Ok(ManyToOneDecorator {
        entity,
        join_column,
        cascade,
        fetch,
    })
}

/// Parse many-to-many decorator arguments
pub fn parse_many_to_many_decorator(
    decorator: &Decorator,
) -> Result<ManyToManyDecorator, CompileError> {
    let mut entity = String::new();
    let mut join_table = String::new();
    let mut join_column = Option::<String>::None;
    let mut inverse_join_column = Option::<String>::None;
    let mut cascade = Vec::new();
    let mut fetch = "lazy".to_string();

    for arg in &decorator.arguments {
        if let Expression::Binary(BinaryExpression { left, operator, right, .. }) = arg {
            if operator == "=" {
                if let Expression::Identifier(key) = left.as_ref() {
                    match key.name.as_str() {
                        "entity" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    entity = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for entity".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        "join_table" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    join_table = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for join_table".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        "join_column" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    join_column = Some(value.value.as_str().unwrap().to_string());
                                }
                            }
                        }
                        "inverse_join_column" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    inverse_join_column = Some(value.value.as_str().unwrap().to_string());
                                }
                            }
                        }
                        "cascade" => {
                            // Handle array of strings
                            if let Expression::ArrayLiteral(array) = right.as_ref() {
                                for item in &array.elements {
                                    if let Expression::Literal(value) = item {
                                        if value.kind == "string" {
                                            cascade.push(value.value.as_str()
                                                .ok_or_else(|| CompileError::Parse(ParseError {
                                                    message: "Invalid string value in cascade array".to_string(),
                                                    location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                                    expected: vec!["string".to_string()],
                                                }))?
                                                .to_string());
                                        }
                                    }
                                }
                            }
                        }
                        "fetch" => {
                            if let Expression::Literal(value) = right.as_ref() {
                                if value.kind == "string" {
                                    fetch = value.value.as_str()
                                        .ok_or_else(|| CompileError::Parse(ParseError {
                                            message: "Invalid string value for fetch".to_string(),
                                            location: SourceLocation::new(value.node.range.start.line, value.node.range.start.column, 0),
                                            expected: vec!["string".to_string()],
                                        }))?
                                        .to_string();
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::Parse(ParseError {
                                message: format!("Unknown manyToMany decorator parameter: {}", key.name),
                                location: SourceLocation::new(key.node.range.start.line, key.node.range.start.column, 0),
                                expected: vec!["entity".to_string(), "join_table".to_string(), "join_column".to_string(), "inverse_join_column".to_string(), "cascade".to_string(), "fetch".to_string()],
                            }));
                        }
                    }
                }
            }
        }
    }

    if entity.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "manyToMany decorator requires entity parameter".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["entity".to_string()],
        }));
    }

    if join_table.is_empty() {
        return Err(CompileError::Parse(ParseError {
            message: "manyToMany decorator requires join_table parameter".to_string(),
            location: SourceLocation::new(decorator.range.start.line, decorator.range.start.column, 0),
            expected: vec!["join_table".to_string()],
        }));
    }

    Ok(ManyToManyDecorator {
        entity,
        join_table,
        join_column: join_column.unwrap_or_else(|| "entity1_id".to_string()),
        inverse_join_column: inverse_join_column.unwrap_or_else(|| "entity2_id".to_string()),
        cascade,
        fetch,
    })
}

/// Generate ORM code for a struct with ORM decorators
pub fn generate_orm_code(
    decorators: &[OrmDecorator],
    struct_name: &str,
    fields: &[(&str, &str)], // (field_name, field_type)
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Check if this is an entity
    let entity_decorator = decorators.iter().find_map(|d| {
        if let OrmDecorator::Entity(entity) = d {
            Some(entity)
        } else {
            None
        }
    });

    if let Some(entity_config) = entity_decorator {
        // Generate entity implementation
        code.push_str(&generate_entity_implementation(struct_name, entity_config, decorators, fields)?);
    }

    Ok(code)
}

/// Generate entity implementation code
fn generate_entity_implementation(
    struct_name: &str,
    entity_config: &EntityDecorator,
    decorators: &[OrmDecorator],
    fields: &[(&str, &str)],
) -> Result<String, CompileError> {
    let mut code = String::new();

    // Generate imports
    code.push_str("use vela_orm::{Entity, EntityMetadata, FieldMetadata, RelationMetadata, RelationType, FetchType, CascadeType};\n");
    code.push_str("use vela_orm::entity::EntityManager;\n");
    code.push_str("use std::collections::HashMap;\n\n");

    // Generate Entity implementation
    code.push_str(&format!("impl Entity for {} {{\n", struct_name));

    // Generate metadata method
    code.push_str("    fn metadata() -> EntityMetadata {\n");
    code.push_str(&format!("        let mut fields = HashMap::new();\n"));
    code.push_str(&format!("        let mut relations = HashMap::new();\n\n"));

    // Process fields
    for (field_name, field_type) in fields {
        // Check if field has decorators
        let field_decorators: Vec<&OrmDecorator> = decorators.iter()
            .filter(|d| matches!(d, OrmDecorator::Id(_) | OrmDecorator::Column(_) | OrmDecorator::OneToMany(_) | OrmDecorator::ManyToOne(_) | OrmDecorator::ManyToMany(_)))
            .collect();

        if let Some(column_decorator) = field_decorators.iter().find_map(|d| {
            if let OrmDecorator::Column(col) = d {
                Some(col)
            } else {
                None
            }
        }) {
            // Generate field metadata
            let column_name = column_decorator.name.as_ref().map(|s| s.as_str()).unwrap_or(field_name);
            code.push_str(&format!("        fields.insert(\n"));
            code.push_str(&format!("            \"{}\".to_string(),\n", field_name));
            code.push_str(&format!("            FieldMetadata {{\n"));
            code.push_str(&format!("                name: \"{}\".to_string(),\n", column_name));
            code.push_str(&format!("                field_type: \"{}\".to_string(),\n", field_type));
            code.push_str(&format!("                nullable: {},\n", column_decorator.nullable));
            code.push_str(&format!("                unique: {},\n", column_decorator.unique));
            code.push_str(&format!("                primary_key: {},\n", column_decorator.primary_key));
            code.push_str(&format!("                generated: {},\n", column_decorator.generated));
            if let Some(default) = &column_decorator.default {
                code.push_str(&format!("                default_value: Some(\"{}\".to_string()),\n", default));
            } else {
                code.push_str(&format!("                default_value: None,\n"));
            }
            code.push_str(&format!("            }},\n"));
            code.push_str(&format!("        );\n\n"));
        }

        // Handle ID decorator
        if let Some(id_decorator) = field_decorators.iter().find_map(|d| {
            if let OrmDecorator::Id(id) = d {
                Some(id)
            } else {
                None
            }
        }) {
            code.push_str(&format!("        // ID field: {} (generated: {})\n", field_name, id_decorator.generated));
        }

        // Handle relation decorators
        if let Some(relation_decorator) = field_decorators.iter().find_map(|d| {
            match d {
                OrmDecorator::OneToMany(rel) => Some(("one_to_many", rel.entity.as_str(), rel.mapped_by.as_str(), rel.fetch.as_str())),
                OrmDecorator::ManyToOne(rel) => Some(("many_to_one", rel.entity.as_str(), rel.join_column.as_str(), rel.fetch.as_str())),
                OrmDecorator::ManyToMany(rel) => Some(("many_to_many", rel.entity.as_str(), rel.join_table.as_str(), rel.fetch.as_str())),
                _ => None,
            }
        }) {
            let (relation_type, target_entity, join_info, fetch_type) = relation_decorator;
            let fetch_type_enum = match fetch_type {
                "eager" => "FetchType::Eager",
                _ => "FetchType::Lazy",
            };

            code.push_str(&format!("        relations.insert(\n"));
            code.push_str(&format!("            \"{}\".to_string(),\n", field_name));
            code.push_str(&format!("            RelationMetadata {{\n"));
            code.push_str(&format!("                relation_type: RelationType::{},\n", match relation_type {
                "one_to_many" => "OneToMany",
                "many_to_one" => "ManyToOne",
                "many_to_many" => "ManyToMany",
                _ => "OneToMany",
            }));
            code.push_str(&format!("                target_entity: \"{}\".to_string(),\n", target_entity));
            code.push_str(&format!("                join_column: Some(\"{}\".to_string()),\n", join_info));
            code.push_str(&format!("                fetch_type: {},\n", fetch_type_enum));
            code.push_str(&format!("                cascade: vec![], // TODO: Add cascade support\n"));
            code.push_str(&format!("            }},\n"));
            code.push_str(&format!("        );\n\n"));
        }
    }

    // Generate EntityMetadata
    let default_table_name = struct_name.to_lowercase();
    let table_name = entity_config.table.as_ref()
        .unwrap_or(&default_table_name);
    let schema_name = entity_config.schema.as_ref()
        .map(|s| format!("Some(\"{}\".to_string())", s))
        .unwrap_or("None".to_string());

    code.push_str(&format!("        EntityMetadata {{\n"));
    code.push_str(&format!("            name: \"{}\".to_string(),\n", struct_name));
    code.push_str(&format!("            table: \"{}\".to_string(),\n", table_name));
    code.push_str(&format!("            schema: {},\n", schema_name));
    code.push_str(&format!("            fields,\n"));
    code.push_str(&format!("            relations,\n"));
    code.push_str(&format!("        }}\n"));
    code.push_str("    }\n");

    // Generate other required methods (placeholders for now)
    code.push_str("\n    fn find_by_id(db: &vela_orm::Database, id: i64) -> impl std::future::Future<Output = vela_orm::Result<Option<Self>>> + Send {\n");
    code.push_str("        async move { Ok(None) } // TODO: Implement\n");
    code.push_str("    }\n\n");

    code.push_str("    fn find_all(db: &vela_orm::Database) -> impl std::future::Future<Output = vela_orm::Result<Vec<Self>>> + Send {\n");
    code.push_str("        async move { Ok(vec![]) } // TODO: Implement\n");
    code.push_str("    }\n\n");

    code.push_str("    fn save(&self, db: &vela_orm::Database) -> impl std::future::Future<Output = vela_orm::Result<()>> + Send {\n");
    code.push_str("        async move { Ok(()) } // TODO: Implement\n");
    code.push_str("    }\n\n");

    code.push_str("    fn delete(&self, db: &vela_orm::Database) -> impl std::future::Future<Output = vela_orm::Result<()>> + Send {\n");
    code.push_str("        async move { Ok(()) } // TODO: Implement\n");
    code.push_str("    }\n");

    code.push_str("}\n\n");

    Ok(code)
}
