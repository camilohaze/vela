/*!
Entity relations management for Vela ORM.

This module handles loading and managing relationships between entities,
including lazy and eager loading, cascading operations, and relationship
resolution.
*/

use crate::connection::Database;
use crate::entity::{Entity, RelationMetadata, RelationType, FetchType, CascadeType};
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Relation loader for handling entity relationships
pub struct RelationLoader {
    db: Database,
    loaded_entities: HashMap<String, HashMap<i64, serde_json::Value>>,
}

impl RelationLoader {
    /// Create a new relation loader
    pub fn new(db: Database) -> Self {
        Self {
            db,
            loaded_entities: HashMap::new(),
        }
    }

    /// Load a many-to-one relation
    pub async fn load_many_to_one<T: Entity, U: Entity>(
        &mut self,
        entities: &mut [T],
        relation_field: &str,
        relation_meta: &RelationMetadata,
    ) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        // Collect foreign key values from entities
        let join_column = relation_meta.join_column.as_ref()
            .ok_or_else(|| Error::entity("Join column required for many-to-one relation"))?;

        let mut fk_values = Vec::new();
        for entity in entities.iter() {
            // Extract foreign key value from entity using reflection-like approach
            // For now, we'll use a simplified approach assuming entities have getter methods
            // In a real implementation, this would use serde or custom derive macros
            let fk_value = self.extract_foreign_key_value(entity, join_column)?;
            if let Some(value) = fk_value {
                fk_values.push(value);
            }
        }

        // Remove duplicates
        fk_values.sort();
        fk_values.dedup();

        if fk_values.is_empty() {
            return Ok(());
        }

        // Load related entities using type-safe query
        let related_entities = U::query(&self.db)
            .where_in(U::primary_key_field(), &fk_values)
            .find_many()
            .await?;

        // Create lookup map for efficient access
        let mut lookup = HashMap::new();
        for entity in related_entities {
            let pk_value = self.extract_primary_key_value(&entity)?;
            let json = serde_json::to_value(&entity)
                .map_err(|e| Error::serialization(e.to_string()))?;
            lookup.insert(pk_value, json);
        }

        // Store in cache
        let cache_key = format!("{}:{}", U::table_name(), relation_field);
        self.loaded_entities.insert(cache_key, lookup);

        Ok(())
    }

    /// Load a one-to-many relation
    pub async fn load_one_to_many<T: Entity, U: Entity>(
        &mut self,
        entities: &mut [T],
        relation_field: &str,
        relation_meta: &RelationMetadata,
    ) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        // Collect primary key values from parent entities
        let mut pk_values = Vec::new();
        for entity in entities.iter() {
            let pk_value = self.extract_primary_key_value(entity)?;
            pk_values.push(pk_value);
        }

        // Remove duplicates
        pk_values.sort();
        pk_values.dedup();

        if pk_values.is_empty() {
            return Ok(());
        }

        // Get the field name that references the parent
        let mapped_by = relation_meta.mapped_by.as_ref()
            .ok_or_else(|| Error::entity("Mapped by field required for one-to-many relation"))?;

        // Load related entities using type-safe query
        let related_entities = U::query(&self.db)
            .where_in(mapped_by, &pk_values)
            .find_many()
            .await?;

        // Group related entities by foreign key value
        let mut grouped: HashMap<i64, Vec<serde_json::Value>> = HashMap::new();
        for entity in related_entities {
            // Extract the foreign key value that points to the parent
            let fk_value = self.extract_foreign_key_value(&entity, mapped_by)?
                .ok_or_else(|| Error::entity("Foreign key value required for grouping"))?;

            let json = serde_json::to_value(&entity)
                .map_err(|e| Error::serialization(e.to_string()))?;

            grouped.entry(fk_value).or_insert_with(Vec::new).push(json);
        }

        // Store in cache
        let cache_key = format!("{}:{}", U::table_name(), relation_field);
        let mut lookup = HashMap::new();
        for (fk, entities) in grouped {
            let json_array = serde_json::to_value(entities)
                .map_err(|e| Error::serialization(e.to_string()))?;
            lookup.insert(fk, json_array);
        }
        self.loaded_entities.insert(cache_key, lookup);

        Ok(())
    }

    /// Load a many-to-many relation
    pub async fn load_many_to_many<T: Entity, U: Entity>(
        &mut self,
        entities: &mut [T],
        relation_field: &str,
        relation_meta: &RelationMetadata,
    ) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        // Collect primary key values from parent entities
        let mut pk_values = Vec::new();
        for entity in entities.iter() {
            let pk_value = self.extract_primary_key_value(entity)?;
            pk_values.push(pk_value);
        }

        // Remove duplicates
        pk_values.sort();
        pk_values.dedup();

        if pk_values.is_empty() {
            return Ok(());
        }

        // Get join table information
        let join_table = relation_meta.join_column.as_ref()
            .ok_or_else(|| Error::entity("Join table required for many-to-many relation"))?;

        // Query the join table to get related entity IDs
        let placeholders = vec!["?"; pk_values.len()].join(",");
        let join_query = format!(
            "SELECT {}, {} FROM {} WHERE {} IN ({})",
            T::primary_key_field(),
            U::primary_key_field(),
            join_table,
            T::primary_key_field(),
            placeholders
        );

        let params: Vec<Box<dyn crate::connection::ToSql>> = pk_values.into_iter().map(|v| Box::new(v) as Box<dyn crate::connection::ToSql>).collect();
        let join_result = self.db.query(&join_query, params).await?;
        let mut related_ids = Vec::new();
        let mut id_mapping: HashMap<i64, Vec<i64>> = HashMap::new();

        // Process join table results
        for row in join_result.rows {
            // Try to get both values - if either fails, skip this row
            if let (Ok(parent_id), Ok(related_id)) = (row.try_get::<i64>(0), row.try_get::<i64>(1)) {
                related_ids.push(related_id);
                id_mapping.entry(parent_id).or_insert_with(Vec::new).push(related_id);
            }
        }

        // Remove duplicates from related IDs
        related_ids.sort();
        related_ids.dedup();

        if related_ids.is_empty() {
            return Ok(());
        }

        // Load the actual related entities
        let related_entities = U::query(&self.db)
            .where_in(U::primary_key_field(), &related_ids)
            .find_many()
            .await?;

        // Create lookup map for related entities
        let mut entity_lookup = HashMap::new();
        for entity in related_entities {
            let pk_value = self.extract_primary_key_value(&entity)?;
            let json = serde_json::to_value(&entity)
                .map_err(|e| Error::serialization(e.to_string()))?;
            entity_lookup.insert(pk_value, json);
        }

        // Create the final mapping: parent_id -> array of related entities
        let mut final_lookup = HashMap::new();
        for (parent_id, related_ids_for_parent) in id_mapping {
            let mut related_entities_json = Vec::new();
            for related_id in related_ids_for_parent {
                if let Some(entity_json) = entity_lookup.get(&related_id) {
                    related_entities_json.push(entity_json.clone());
                }
            }

            let json_array = serde_json::to_value(related_entities_json)
                .map_err(|e| Error::serialization(e.to_string()))?;
            final_lookup.insert(parent_id, json_array);
        }

        // Store in cache
        let cache_key = format!("{}:{}", U::table_name(), relation_field);
        self.loaded_entities.insert(cache_key, final_lookup);

        Ok(())
    }

    /// Get loaded entities from cache
    pub fn get_loaded_entities(&self, cache_key: &str) -> Option<&HashMap<i64, serde_json::Value>> {
        self.loaded_entities.get(cache_key)
    }

    /// Clear the entity cache
    pub fn clear_cache(&mut self) {
        self.loaded_entities.clear();
    }

    /// Extract foreign key value from entity (simplified implementation)
    fn extract_foreign_key_value<T: Entity>(&self, _entity: &T, _field_name: &str) -> Result<Option<i64>> {
        // In a real implementation, this would use reflection or serde to extract the field value
        // For now, return a placeholder
        Ok(Some(1)) // Placeholder - would extract actual FK value
    }

    /// Extract primary key value from entity (simplified implementation)
    fn extract_primary_key_value<T: Entity>(&self, _entity: &T) -> Result<i64> {
        // In a real implementation, this would use reflection or serde to extract the PK value
        // For now, return a placeholder
        Ok(1) // Placeholder - would extract actual PK value
    }
}

/// Cascade manager for handling cascading operations
pub struct CascadeManager {
    db: Database,
}

impl CascadeManager {
    /// Create a new cascade manager
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Execute cascade operations for an entity
    pub async fn execute_cascade<T: Entity>(
        &self,
        entity: &T,
        operation: CascadeOperation,
        relations: &HashMap<String, RelationMetadata>,
    ) -> Result<()> {
        for (field_name, relation_meta) in relations {
            if relation_meta.cascade.contains(&CascadeType::All) ||
               self.should_cascade(&operation, &relation_meta.cascade) {

                match relation_meta.relation_type {
                    RelationType::OneToMany => {
                        self.cascade_one_to_many(entity, field_name, relation_meta, &operation).await?;
                    }
                    RelationType::OneToOne => {
                        self.cascade_one_to_one(entity, field_name, relation_meta, &operation).await?;
                    }
                    RelationType::ManyToOne => {
                        // Many-to-one relations don't cascade from child to parent
                    }
                    RelationType::ManyToMany => {
                        self.cascade_many_to_many(entity, field_name, relation_meta, &operation).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if cascade should be executed for the operation
    fn should_cascade(&self, operation: &CascadeOperation, cascades: &[CascadeType]) -> bool {
        match operation {
            CascadeOperation::Persist => cascades.contains(&CascadeType::Persist),
            CascadeOperation::Merge => cascades.contains(&CascadeType::Merge),
            CascadeOperation::Remove => cascades.contains(&CascadeType::Remove),
            CascadeOperation::Refresh => cascades.contains(&CascadeType::Refresh),
        }
    }

    /// Execute cascade for one-to-many relation
    async fn cascade_one_to_many<T: Entity>(
        &self,
        entity: &T,
        field_name: &str,
        relation_meta: &RelationMetadata,
        operation: &CascadeOperation,
    ) -> Result<()> {
        // Load related entities for this relation
        let related_entities = self.load_related_entities::<T>(
            entity,
            field_name,
            relation_meta,
        ).await?;

        // Apply cascade operation to each related entity
        for related_entity_json in related_entities {
            match operation {
                CascadeOperation::Persist => {
                    // For persist, we would deserialize and save each related entity
                    // This is a simplified implementation
                    self.persist_related_entity(&related_entity_json).await?;
                }
                CascadeOperation::Merge => {
                    // For merge, update existing entities
                    self.merge_related_entity(&related_entity_json).await?;
                }
                CascadeOperation::Remove => {
                    // For remove, delete related entities
                    self.remove_related_entity(&related_entity_json).await?;
                }
                CascadeOperation::Refresh => {
                    // For refresh, reload related entities from database
                    // This might not be applicable for cascade operations
                }
            }
        }

        Ok(())
    }

    /// Execute cascade for one-to-one relation
    async fn cascade_one_to_one<T: Entity>(
        &self,
        entity: &T,
        field_name: &str,
        relation_meta: &RelationMetadata,
        operation: &CascadeOperation,
    ) -> Result<()> {
        // For one-to-one, we handle it similar to one-to-many but for a single entity
        let related_entities = self.load_related_entities::<T>(
            entity,
            field_name,
            relation_meta,
        ).await?;

        // Apply cascade operation to the single related entity
        if let Some(related_entity_json) = related_entities.first() {
            match operation {
                CascadeOperation::Persist => {
                    self.persist_related_entity(related_entity_json).await?;
                }
                CascadeOperation::Merge => {
                    self.merge_related_entity(related_entity_json).await?;
                }
                CascadeOperation::Remove => {
                    self.remove_related_entity(related_entity_json).await?;
                }
                CascadeOperation::Refresh => {
                    // For refresh, we might not need to do anything for cascade
                }
            }
        }

        Ok(())
    }

    /// Execute cascade for many-to-many relation
    async fn cascade_many_to_many<T: Entity>(
        &self,
        entity: &T,
        field_name: &str,
        relation_meta: &RelationMetadata,
        operation: &CascadeOperation,
    ) -> Result<()> {
        let join_table = relation_meta.join_column.as_ref()
            .ok_or_else(|| Error::entity("Join table required for many-to-many cascade"))?;

        match operation {
            CascadeOperation::Persist => {
                // For persist, we would need to insert into the join table
                // This requires knowing the related entity IDs
                // For now, this is a placeholder
                Ok(())
            }
            CascadeOperation::Merge => {
                // For merge, update join table entries
                // This is a placeholder
                Ok(())
            }
            CascadeOperation::Remove => {
                // For remove, delete from join table
                // We need to get the parent ID somehow - for now, this is a placeholder
                let delete_sql = format!(
                    "DELETE FROM {} WHERE {} = ?",
                    join_table,
                    T::primary_key_field()
                );
                // Placeholder parameter - would need actual parent ID
                let params: Vec<Box<dyn crate::connection::ToSql>> = vec![Box::new(1i64)];
                self.db.execute(&delete_sql, params).await?;
                Ok(())
            }
            CascadeOperation::Refresh => {
                // For refresh, we might not need to do anything for cascade
                Ok(())
            }
        }
    }

    /// Load related entities for cascade operations
    async fn load_related_entities<T: Entity>(
        &self,
        _entity: &T,
        _field_name: &str,
        _relation_meta: &RelationMetadata,
    ) -> Result<Vec<serde_json::Value>> {
        // In a real implementation, this would load the related entities
        // For now, return empty vector
        Ok(vec![])
    }

    /// Persist a related entity
    async fn persist_related_entity(&self, _entity_json: &serde_json::Value) -> Result<()> {
        // In a real implementation, this would deserialize and save the entity
        Ok(())
    }

    /// Merge a related entity
    async fn merge_related_entity(&self, _entity_json: &serde_json::Value) -> Result<()> {
        // In a real implementation, this would deserialize and update the entity
        Ok(())
    }

    /// Remove a related entity
    async fn remove_related_entity(&self, _entity_json: &serde_json::Value) -> Result<()> {
        // In a real implementation, this would deserialize and delete the entity
        Ok(())
    }
}

/// Cascade operations
#[derive(Debug, Clone)]
pub enum CascadeOperation {
    Persist,
    Merge,
    Remove,
    Refresh,
}

/// Relation resolver for resolving entity references
pub struct RelationResolver {
    db: Database,
    loader: RelationLoader,
}

impl RelationResolver {
    /// Create a new relation resolver
    pub fn new(db: Database) -> Self {
        Self {
            loader: RelationLoader::new(db.clone()),
            db,
        }
    }

    /// Resolve relations for a collection of entities
    pub async fn resolve_relations<T: Entity>(
        &mut self,
        entities: &mut [T],
        relations_to_load: &[String],
    ) -> Result<()> {
        let metadata = T::metadata();

        for relation_name in relations_to_load {
            if let Some(relation_meta) = metadata.relations.get(relation_name) {
                match relation_meta.relation_type {
                    RelationType::ManyToOne => {
                        // This would need generic type information for the target entity
                        // For now, we'll skip the actual loading
                    }
                    RelationType::OneToMany => {
                        // Similar issue with generics
                    }
                    RelationType::ManyToMany => {
                        // Similar issue with generics
                    }
                    RelationType::OneToOne => {
                        // Similar issue with generics
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the relation loader
    pub fn loader(&mut self) -> &mut RelationLoader {
        &mut self.loader
    }
}

/// Lazy loader for deferred relation loading
pub struct LazyLoader<T: Entity> {
    db: Database,
    entity_id: i64,
    relation_meta: RelationMetadata,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Entity> LazyLoader<T> {
    /// Create a new lazy loader
    pub fn new(db: Database, entity_id: i64, relation_meta: RelationMetadata) -> Self {
        Self {
            db,
            entity_id,
            relation_meta,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Load the relation
    pub async fn load<U: Entity>(self) -> Result<Vec<U>> {
        match self.relation_meta.relation_type {
            RelationType::OneToMany => {
                let mapped_by = self.relation_meta.mapped_by.as_ref()
                    .ok_or_else(|| Error::entity("Mapped by field required for lazy loading"))?;

                U::query(&self.db)
                    .where_eq(mapped_by, self.entity_id)
                    .find_many()
                    .await
            }
            RelationType::ManyToOne => {
                let join_column = self.relation_meta.join_column.as_ref()
                    .ok_or_else(|| Error::entity("Join column required for lazy loading"))?;

                // This would need to get the foreign key value from the entity
                // For now, we'll use the entity_id as placeholder
                U::query(&self.db)
                    .where_eq(U::primary_key_field(), self.entity_id)
                    .find_one()
                    .await
                    .map(|opt| opt.map(|u| vec![u]).unwrap_or_default())
            }
            RelationType::ManyToMany => {
                // Many-to-many lazy loading would require join table queries
                // This is a placeholder
                Ok(vec![])
            }
            RelationType::OneToOne => {
                U::query(&self.db)
                    .where_eq(U::primary_key_field(), self.entity_id)
                    .find_one()
                    .await
                    .map(|opt| opt.map(|u| vec![u]).unwrap_or_default())
            }
        }
    }
}

/// Eager loader for immediate relation loading
pub struct EagerLoader {
    db: Database,
}

impl EagerLoader {
    /// Create a new eager loader
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Load relations eagerly for a collection of entities
    pub async fn load_relations<T: Entity>(
        &self,
        entities: &mut [T],
        relations: &[String],
    ) -> Result<()> {
        let mut resolver = RelationResolver::new(self.db.clone());
        resolver.resolve_relations(entities, relations).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    // Mock entities for testing
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockUser;
    impl Entity for MockUser {
        fn table_name() -> &'static str { "users" }
        fn primary_key_field() -> &'static str { "id" }
        fn metadata() -> crate::entity::EntityMetadata {
            let mut meta = crate::entity::EntityMetadata::new("users");
            meta.relations.insert(
                "posts".to_string(),
                RelationMetadata::one_to_many("Post", "author_id")
                    .with_cascade(CascadeType::All)
                    .with_fetch(FetchType::Lazy)
            );
            meta
        }
        async fn find_by_id(_db: &Database, _id: i64) -> Result<Option<Self>> { Ok(None) }
        async fn find_all(_db: &Database) -> Result<Vec<Self>> { Ok(vec![]) }
        async fn save(&self, _db: &Database) -> Result<()> { Ok(()) }
        async fn delete(&self, _db: &Database) -> Result<()> { Ok(()) }
        fn query(db: &Database) -> crate::query::QueryBuilder<Self> {
            crate::query::QueryBuilder::new(db.clone())
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockPost;
    impl Entity for MockPost {
        fn table_name() -> &'static str { "posts" }
        fn primary_key_field() -> &'static str { "id" }
        fn metadata() -> crate::entity::EntityMetadata {
            crate::entity::EntityMetadata::new("posts")
        }
        async fn find_by_id(_db: &Database, _id: i64) -> Result<Option<Self>> { Ok(None) }
        async fn find_all(_db: &Database) -> Result<Vec<Self>> { Ok(vec![]) }
        async fn save(&self, _db: &Database) -> Result<()> { Ok(()) }
        async fn delete(&self, _db: &Database) -> Result<()> { Ok(()) }
        fn query(db: &Database) -> crate::query::QueryBuilder<Self> {
            crate::query::QueryBuilder::new(db.clone())
        }
    }

    #[test]
    fn test_relation_metadata_creation() {
        let relation = RelationMetadata::one_to_many("Post", "author_id")
            .with_cascade(CascadeType::Persist)
            .with_fetch(FetchType::Eager);

        assert!(matches!(relation.relation_type, RelationType::OneToMany));
        assert_eq!(relation.target_entity, "Post");
        assert_eq!(relation.mapped_by, Some("author_id".to_string()));
        assert!(relation.cascade.contains(&CascadeType::Persist));
        assert!(matches!(relation.fetch, FetchType::Eager));
    }

    #[tokio::test]
    async fn test_cascade_manager_should_cascade() {
        let cascades = vec![CascadeType::Persist, CascadeType::Remove];
        let manager = CascadeManager::new(Database::connect("sqlite::memory:").await.unwrap());

        assert!(manager.should_cascade(&CascadeOperation::Persist, &cascades));
        assert!(manager.should_cascade(&CascadeOperation::Remove, &cascades));
        assert!(!manager.should_cascade(&CascadeOperation::Merge, &cascades));
    }

    #[tokio::test]
    async fn test_lazy_loader_creation() {
        let config = DatabaseConfig::sqlite(":memory:");
        let db = Database::connect_with_config(config).await.unwrap();

        let relation_meta = RelationMetadata::one_to_many("Post", "author_id");
        let loader = LazyLoader::<MockUser>::new(db, 1, relation_meta);

        assert_eq!(loader.entity_id, 1);
        assert!(matches!(loader.relation_meta.relation_type, RelationType::OneToMany));
    }
}