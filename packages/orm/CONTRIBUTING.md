# Contributing to Vela ORM

Thank you for your interest in contributing to the Vela ORM! This document provides guidelines and information for contributors.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/0/code_of_conduct/). By participating, you are expected to uphold this code.

## How to Contribute

### 1. Development Setup

```bash
# Clone the repository
git clone https://github.com/velalang/vela.git
cd vela/packages/orm

# Install dependencies
cargo build

# Run tests
cargo test

# Run with specific features
cargo test --features postgres
cargo test --features mysql
```

### 2. Development Workflow

1. **Choose an issue** from the [issue tracker](https://github.com/velalang/vela/issues)
2. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** following the coding standards
4. **Add tests** for your changes
5. **Run the test suite**:
   ```bash
   cargo test
   cargo test --doc
   cargo clippy
   cargo fmt --check
   ```
6. **Update documentation** if needed
7. **Commit your changes**:
   ```bash
   git commit -m "feat: add your feature description"
   ```
8. **Push and create a pull request**

### 3. Coding Standards

#### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Write comprehensive documentation comments (`///`)
- Use meaningful variable and function names

#### Error Handling

```rust
// ✅ Good: Use custom error types
#[derive(Debug, thiserror::Error)]
pub enum OrmError {
    #[error("Connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Entity not found: {0}")]
    NotFound(String),
}

// ❌ Bad: Use generic errors
pub fn find_user(id: i64) -> Result<User, Box<dyn std::error::Error>> {
    // ...
}
```

#### Async Code

```rust
// ✅ Good: Use async traits properly
#[async_trait::async_trait]
pub trait Repository<T> {
    async fn find_by_id(&self, id: i64) -> Result<Option<T>>;
    async fn save(&self, entity: T) -> Result<T>;
}

// ✅ Good: Proper error propagation
pub async fn create_user(&self, user: User) -> Result<User> {
    let mut tx = self.db.begin().await?;
    let result = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        user.name,
        user.email
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(result)
}
```

### 4. Testing

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_creation() {
        let db = setup_test_db().await;
        let user = User { /* ... */ };

        let saved = user.save(&db).await.unwrap();
        assert_eq!(saved.name, user.name);
    }
}
```

#### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_user_crud(pool: PgPool) {
        let db = Database::from_pool(pool);
        // Test full CRUD operations
    }
}
```

#### Test Database Setup

```rust
async fn setup_test_db() -> Database {
    let config = DatabaseConfig::Sqlite {
        database: ":memory:".to_string(),
        ..Default::default()
    };
    Database::connect_with_config(config).await.unwrap()
}
```

### 5. Documentation

#### Code Documentation

```rust
/// Represents a user in the system.
///
/// # Examples
///
/// ```
/// use vela_orm::User;
///
/// let user = User {
///     id: 1,
///     name: "John Doe".to_string(),
///     email: "john@example.com".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct User {
    /// The unique identifier for the user
    pub id: i64,

    /// The user's full name
    pub name: String,

    /// The user's email address
    pub email: String,
}
```

#### API Documentation

- Document all public APIs
- Include examples in doc comments
- Explain error conditions
- Document performance characteristics

### 6. Performance Guidelines

#### Database Queries

- Use prepared statements
- Avoid N+1 query problems
- Use appropriate indexes
- Batch operations when possible

#### Memory Usage

- Avoid unnecessary allocations
- Use streaming for large result sets
- Implement proper resource cleanup

#### Connection Pooling

- Configure appropriate pool sizes
- Implement connection health checks
- Handle connection timeouts properly

### 7. Security Considerations

#### SQL Injection Prevention

```rust
// ✅ Good: Use parameterized queries
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = $1",
    user_id
)
.fetch_one(&db)
.await?;

// ❌ Bad: String concatenation
let query = format!("SELECT * FROM users WHERE id = {}", user_id);
```

#### Data Validation

```rust
// ✅ Good: Validate input
pub fn create_user(name: String, email: String) -> Result<User> {
    if name.is_empty() {
        return Err(OrmError::Validation("Name cannot be empty".to_string()));
    }
    if !email.contains('@') {
        return Err(OrmError::Validation("Invalid email format".to_string()));
    }
    // ...
}
```

### 8. Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(entities): add support for composite primary keys

fix(query): prevent SQL injection in raw queries

docs(api): update migration guide

test(relations): add tests for many-to-many relations
```

### 9. Pull Request Process

1. **Title**: Use conventional commit format
2. **Description**: Include:
   - What changes were made
   - Why they were made
   - How to test the changes
   - Any breaking changes
3. **Labels**: Add appropriate labels
4. **Assignees**: Assign to appropriate reviewers
5. **Milestone**: Link to project milestone

### 10. Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. Publish to crates.io
5. Create GitHub release

### 11. Getting Help

- **Issues**: Use GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions
- **Discord**: Join our Discord community for real-time help

### 12. Recognition

Contributors will be recognized in:
- `CHANGELOG.md` for significant changes
- GitHub release notes
- Project documentation

Thank you for contributing to Vela ORM! 🚀