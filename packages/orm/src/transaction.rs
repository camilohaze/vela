/*!
Database transactions for Vela ORM.

This module provides transaction management with automatic rollback,
savepoints, and transaction isolation levels.
*/

use crate::connection::{Database, ToSql};
use crate::error::{Error, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Transaction isolation levels
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    /// Read Uncommitted - lowest isolation level
    ReadUncommitted,
    /// Read Committed - default for most databases
    ReadCommitted,
    /// Repeatable Read - prevents non-repeatable reads
    RepeatableRead,
    /// Serializable - highest isolation level
    Serializable,
}

impl IsolationLevel {
    /// Convert to SQL string
    pub fn as_sql(&self) -> &'static str {
        match self {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        }
    }
}

/// Transaction manager for handling database transactions
pub struct TransactionManager {
    db: Database,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Execute a function within a transaction
    pub async fn execute<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut crate::connection::Transaction) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut tx = self.db.transaction().await?;
        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Execute a function within a transaction with custom isolation level
    pub async fn execute_with_isolation<F, Fut, T>(
        &self,
        isolation_level: IsolationLevel,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut crate::connection::Transaction) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut tx = self.db.transaction().await?;

        // Set isolation level
        let sql = format!("SET TRANSACTION ISOLATION LEVEL {}", isolation_level.as_sql());
        tx.execute(&sql, vec![]).await?;

        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Execute a function within a read-only transaction
    pub async fn execute_readonly<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut crate::connection::Transaction) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut tx = self.db.transaction().await?;

        // Set read-only
        tx.execute("SET TRANSACTION READ ONLY", vec![]).await?;

        let result = f(&mut tx).await;

        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }
}

/// Enhanced transaction with savepoints and nested transactions
#[derive(Clone)]
pub struct EnhancedTransaction {
    db: Database,
    savepoints: Vec<String>,
    committed: bool,
    rolled_back: bool,
}

impl EnhancedTransaction {
    /// Create a new enhanced transaction
    pub async fn new(db: Database) -> Result<Self> {
        db.execute("BEGIN", vec![]).await?;

        Ok(Self {
            db,
            savepoints: Vec::new(),
            committed: false,
            rolled_back: false,
        })
    }

    /// Create a savepoint
    pub async fn savepoint(&mut self, name: &str) -> Result<()> {
        if self.committed || self.rolled_back {
            return Err(Error::transaction("Transaction already finished"));
        }

        let sql = format!("SAVEPOINT {}", name);
        self.db.execute(&sql, vec![]).await?;
        self.savepoints.push(name.to_string());

        Ok(())
    }

    /// Rollback to a savepoint
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> Result<()> {
        if self.committed || self.rolled_back {
            return Err(Error::transaction("Transaction already finished"));
        }

        // Check if savepoint exists
        let position = self.savepoints.iter().rposition(|s| s == name)
            .ok_or_else(|| Error::transaction(format!("Savepoint '{}' not found", name)))?;

        let sql = format!("ROLLBACK TO SAVEPOINT {}", name);
        self.db.execute(&sql, vec![]).await?;

        // Remove savepoints after the rolled back one
        self.savepoints.truncate(position + 1);

        Ok(())
    }

    /// Release a savepoint
    pub async fn release_savepoint(&mut self, name: &str) -> Result<()> {
        if self.committed || self.rolled_back {
            return Err(Error::transaction("Transaction already finished"));
        }

        let sql = format!("RELEASE SAVEPOINT {}", name);
        self.db.execute(&sql, vec![]).await?;

        // Remove the savepoint from our tracking
        if let Some(position) = self.savepoints.iter().position(|s| s == name) {
            self.savepoints.remove(position);
        }

        Ok(())
    }

    /// Execute a query within the transaction
    pub async fn query(&mut self, sql: &str, params: Vec<Box<dyn ToSql>>) -> Result<crate::connection::QueryResult> {
        if self.committed || self.rolled_back {
            return Err(Error::transaction("Transaction already finished"));
        }
        self.db.query(sql, params).await
    }

    /// Execute a command within the transaction
    pub async fn execute(&mut self, sql: &str, params: Vec<Box<dyn ToSql>>) -> Result<u64> {
        if self.committed || self.rolled_back {
            return Err(Error::transaction("Transaction already finished"));
        }
        self.db.execute(sql, params).await
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> Result<()> {
        if self.committed {
            return Err(Error::transaction("Transaction already committed"));
        }
        if self.rolled_back {
            return Err(Error::transaction("Transaction already rolled back"));
        }

        self.db.execute("COMMIT", vec![]).await?;
        self.committed = true;

        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> Result<()> {
        if self.committed {
            return Err(Error::transaction("Transaction already committed"));
        }
        if self.rolled_back {
            return Ok(()); // Already rolled back
        }

        self.db.execute("ROLLBACK", vec![]).await?;
        self.rolled_back = true;

        Ok(())
    }

    /// Check if the transaction is active
    pub fn is_active(&self) -> bool {
        !self.committed && !self.rolled_back
    }

    /// Get the current savepoints
    pub fn savepoints(&self) -> &[String] {
        &self.savepoints
    }
}

impl Drop for EnhancedTransaction {
    fn drop(&mut self) {
        // In a real implementation, you might want to rollback here
        // But since this is async, we can't do it in drop
        // Instead, we rely on the user to properly commit/rollback
    }
}

/// Transaction context for dependency injection
pub struct TransactionContext {
    transaction: Arc<Mutex<Option<EnhancedTransaction>>>,
}

impl TransactionContext {
    /// Create a new transaction context
    pub fn new() -> Self {
        Self {
            transaction: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the current transaction
    pub async fn set_transaction(&self, tx: EnhancedTransaction) {
        *self.transaction.lock().await = Some(tx);
    }

    /// Get the current transaction
    pub async fn get_transaction(&self) -> Option<Arc<Mutex<EnhancedTransaction>>> {
        if let Some(tx) = self.transaction.lock().await.as_ref() {
            Some(Arc::new(Mutex::new(tx.clone())))
        } else {
            None
        }
    }

    /// Clear the current transaction
    pub async fn clear_transaction(&self) {
        *self.transaction.lock().await = None;
    }

    /// Check if there's an active transaction
    pub async fn has_active_transaction(&self) -> bool {
        self.transaction.lock().await.is_some()
    }
}

impl Default for TransactionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction scope for automatic cleanup
pub struct TransactionScope<'a> {
    context: &'a TransactionContext,
}

impl<'a> TransactionScope<'a> {
    /// Create a new transaction scope
    pub fn new(context: &'a TransactionContext) -> Self {
        Self { context }
    }

    /// Execute a function within a transaction scope
    pub async fn execute<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut EnhancedTransaction) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let tx = EnhancedTransaction::new(self.context.transaction.lock().await.as_ref()
            .ok_or_else(|| Error::transaction("No database connection available"))?
            .db.clone()
        ).await?;

        self.context.set_transaction(tx).await;

        // This is a simplified implementation
        // In practice, you'd want to handle the transaction lifecycle properly
        Err(Error::transaction("Transaction scope not fully implemented"))
    }
}

impl Drop for TransactionScope<'_> {
    fn drop(&mut self) {
        // Cleanup would happen here in a real implementation
    }
}

/// Transaction options for customizing transaction behavior
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    /// Isolation level
    pub isolation_level: Option<IsolationLevel>,
    /// Read-only transaction
    pub read_only: bool,
    /// Transaction timeout in seconds
    pub timeout: Option<u64>,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation_level: None,
            read_only: false,
            timeout: None,
        }
    }
}

impl TransactionOptions {
    /// Create options with a specific isolation level
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = Some(level);
        self
    }

    /// Make the transaction read-only
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Set transaction timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Some(seconds);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[test]
    fn test_isolation_level_sql() {
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
    }

    #[test]
    fn test_transaction_options() {
        let options = TransactionOptions::default()
            .with_isolation_level(IsolationLevel::RepeatableRead)
            .read_only()
            .with_timeout(30);

        assert!(matches!(options.isolation_level, Some(IsolationLevel::RepeatableRead)));
        assert!(options.read_only);
        assert_eq!(options.timeout, Some(30));
    }

    #[tokio::test]
    async fn test_transaction_context() {
        let context = TransactionContext::new();

        assert!(!context.has_active_transaction().await);

        // In a real test, we'd create a transaction and set it
        // But for this test, we'll just check the basic functionality
        context.clear_transaction().await;
        assert!(!context.has_active_transaction().await);
    }

    #[tokio::test]
    async fn test_enhanced_transaction_savepoints() {
        let config = DatabaseConfig::sqlite(":memory:");
        let db = Database::connect_with_config(config).await.unwrap();
        let mut tx = EnhancedTransaction::new(db).await.unwrap();

        // Create savepoints
        tx.savepoint("sp1").await.unwrap();
        tx.savepoint("sp2").await.unwrap();

        assert_eq!(tx.savepoints(), &["sp1".to_string(), "sp2".to_string()]);
        assert!(tx.is_active());

        // Rollback to savepoint
        tx.rollback_to_savepoint("sp1").await.unwrap();
        assert_eq!(tx.savepoints(), &["sp1".to_string()]);

        // Commit transaction
        assert!(tx.is_active());
        tx.commit().await.unwrap();
        // Note: cannot check !tx.is_active() because commit consumes self
    }
}