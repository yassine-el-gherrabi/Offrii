# Rust Programming Guidelines

This document provides guidelines for writing Rust code in this project.

## Quick Reference

### Pre-commit Checklist
```bash
cargo fmt                                                              # Format code
cargo clippy --workspace --all-features --all-targets -- -Dwarnings   # Lint code
cargo sort -w                                                          # Sort dependencies
cargo machete                                                          # Check for unused dependencies
cargo audit                                                            # Security audit
```

### Running Tests
```bash
# Unit tests
cargo nextest run -E 'kind(lib)' --all-features --workspace

# Integration tests
cargo nextest run -E 'kind(test)' --all-features --workspace

# All tests
cargo nextest run --all-features --workspace
```

## Guidelines for Agentic Contributors

When working as an AI agent on this project, you may need to examine source code from external dependencies to
understand their implementation or debug issues. Here are the standard locations where Rust dependency source code can
be found:

### External Library Source Code Locations

- **Dependencies from crates.io**: Look in `~/.cargo/registry/src/index.crates.io-{hash}/` where `{hash}` can change
  depending on the registry index version
- **Dependencies from Git repositories**: Look in `~/.cargo/git/checkouts/` for Git-based dependencies

These locations contain the actual source code of external crates that your project depends on, which can be invaluable
for understanding API behavior, debugging integration issues, or learning implementation patterns.

## Code Style

The project uses Rust 2024 edition and follows standard Rust coding conventions. It also uses the following lints:

```
[workspace.lints.rust]
# Forbiden unsafe code
unsafe_code = { level = "forbid", priority = -10 }
unused_variables = { level = "deny" }
unused = { level = "deny", priority = -1 }
```

**Important**: Safety-related lints (such as `unsafe_code`) should not be updated or modified to fix safety issues. These lints are intentionally strict to maintain code safety and security standards.

## Identify and Avoid Common Anti-Patterns

Before implementing your plan, check whether any common anti-patterns apply to your context. Refactor or plan around them where needed.

**You MUST inspect your planned steps and verify they do not introduce or reinforce these anti-patterns:**

* **Using .clone() instead of borrowing** — Leads to unnecessary memory allocations and reduced performance. Prefer references (`&T`) when you don't need ownership.
* **Overusing .unwrap()/.expect()** — Causes panics and fragile error handling. Use proper error propagation with `?` operator or pattern matching.
* **Calling .collect() too early** — Prevents lazy and efficient iteration. Keep iterators lazy until you actually need the collection.
* **Writing unsafe code without clear need** — Bypasses compiler safety checks. Only use when absolutely necessary and document invariants thoroughly.
* **Over-abstracting with traits/generics** — Makes code harder to understand and increases compile times. Start simple and refactor when patterns emerge.
* **Relying on global mutable state** — Breaks testability and thread safety. Use dependency injection and explicit state passing instead.
* **Using complex macros unnecessarily** — Makes code opaque and harder to debug. Prefer functions, traits, or simple declarative macros.
* **Ignoring proper lifetime annotations** — Leads to confusing borrow checker errors. Be explicit about lifetimes when needed.
* **Premature optimization** — Complicates code before correctness is verified. Profile first, then optimize hot paths with data.
* **Ignoring the type system** — Fighting the compiler instead of working with it. Use newtype pattern and type-driven design.

## Pre-commit Requirements

Before committing any changes to the repository, you must run the following commands to ensure code quality and
consistency:

### 1. Code Formatting

Format all code using `cargo fmt`:

```
cargo fmt
```

This ensures consistent code formatting across the entire project according to the project's rustfmt configuration.

### 2. Code Linting

Run clippy to lint the code and apply fixes:

```
cargo clippy --workspace --all-features --all-targets -- -Dwarnings
```

This command:

- Runs clippy on the entire workspace (`--workspace`)
- Enables all features (`--all-features`)
- Checks all targets including tests, benches, examples (`--all-targets`)
- Treats all warnings as errors (`-- -Dwarnings`)

### 3. Dependency Sorting

Sort all project dependencies using `cargo sort`:

```
cargo sort -w
```

This ensures that dependencies in all `Cargo.toml` files across the workspace are consistently sorted alphabetically.

### 4. Unused Crate Detection

Check for unused crates using `cargo machete`:

```
cargo machete
```

This command identifies and reports any dependencies that are declared in `Cargo.toml` files but are not actually used
in the code, helping to keep the dependency list clean and reduce build times.

### 5. Security Audit

Check for crate vulnerabilities and unmaintained crates using `cargo audit`:

```
cargo audit
```

This command scans your project's dependencies for known security vulnerabilities and unmaintained crates. Note that
security advisories may not always require immediate action and should not block the development workflow.

## Tracing and Instrumentation

The project uses the `tracing` crate for logging and instrumentation. Use the `#[instrument]` attribute on functions to
automatically trace function calls with their arguments and return values.

### Using the #[instrument] Attribute

```rust
use tracing::instrument;

#[instrument]
async fn handle_message(message: Message) -> Result<(), Error> {
    tracing::info!("Handling message");
    // Function arguments are automatically logged
    // ...
    Ok(())
}

// Skip specific fields from being logged
#[instrument(skip(password))]
async fn authenticate(username: String, password: String) -> Result<Token, AuthError> {
    // password won't be logged
    tracing::info!("Authenticating user");
    // ...
}
```

### Tracing Levels

Use appropriate log levels based on the importance and frequency of the message:

- **error!** - Critical errors that require immediate attention (e.g., service failures, data corruption)
  ```rust
  tracing::error!("Failed to connect to database: {}", err);
  ```

- **warn!** - Warning conditions that should be investigated but don't prevent operation (e.g., deprecated API usage, retry scenarios)
  ```rust
  tracing::warn!("Retrying connection attempt {} of {}", attempt, max_retries);
  ```

- **info!** - General informational messages about application flow (e.g., service started, message processed)
  ```rust
  tracing::info!("Processing message with id: {}", message.id);
  ```

- **debug!** - Detailed diagnostic information useful during development (e.g., variable states, execution paths)
  ```rust
  tracing::debug!("Cache hit for key: {}", key);
  ```

- **trace!** - Very verbose logging for deep debugging (e.g., entering/exiting functions, iteration details)
  ```rust
  tracing::trace!("Entering validation loop with {} items", items.len());
  ```

### Best Practices

- Use structured logging with field syntax for better log parsing:
  ```rust
  tracing::info!(user_id = %user.id, action = "login", "User logged in successfully");
  ```

- Avoid logging sensitive information (passwords, tokens, PII) - use `#[instrument(skip(...))]` when needed
- Use `#[instrument]` on public API boundaries and important business logic functions
- Keep log messages concise and actionable

## Error Handling

The project uses the `thiserror` crate for error handling. Define custom error types using the `#[derive(Error)]` attribute for better error messages and type safety.

### Defining Error Types

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to deserialize message: {0}")]
    DeserializationError(String),

    #[error("Failed to process request for '{resource}': {source}")]
    ProcessingError {
        resource: String,
        source: String,
    },

    #[error("Connection failed")]
    ConnectionError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Field '{field}' is required")]
    MissingField { field: String },

    #[error("Invalid format for '{field}': {reason}")]
    InvalidFormat { field: String, reason: String },

    #[error("Value {value} is out of range [{min}, {max}]")]
    OutOfRange { value: i64, min: i64, max: i64 },
}
```

### Error Propagation

Use the `?` operator for clean error propagation:

```rust
use std::fs;
use std::io;

fn read_config(path: &str) -> Result<Config, io::Error> {
    let content = fs::read_to_string(path)?;  // Automatically propagates io::Error
    let config = parse_config(&content)?;      // Propagates parsing errors
    Ok(config)
}
```

### Error Context with anyhow

For application-level code where you need error context, consider using `anyhow`:

```rust
use anyhow::{Context, Result};

fn load_user_data(user_id: &str) -> Result<UserData> {
    let path = format!("data/{}.json", user_id);
    let content = fs::read_to_string(&path)
        .context(format!("Failed to read user data file: {}", path))?;

    let data = serde_json::from_str(&content)
        .context("Failed to parse user data JSON")?;

    Ok(data)
}
```

### Best Practices

- Use `thiserror` for library code and custom error types
- Use `anyhow` for application code where you need rich error context
- Always use `Result<T, E>` instead of panicking with `unwrap()` or `expect()` in production code
- Provide meaningful error messages that help with debugging
- Use `#[from]` to automatically convert from underlying error types
- Include relevant context in error variants (field names, values, etc.)

## Environment Variables

The project uses environment variables for configuration. Use the `.env` file for local development. The `dotenv` crate
is used to load environment variables from the `.env` file:

```rust
use dotenv::dotenv;

// Load .env file at application startup
dotenv().ok();
```

### Reading Environment Variables

Use `std::env::var` to read environment variables with proper error handling:

```rust
use std::env;

// Required environment variable - panics if not set
let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

// Optional environment variable with default value
let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

// Optional environment variable
let optional_config = env::var("OPTIONAL_CONFIG").ok();
```

## Testing Guidelines

### Test Organization

The project follows Rust's standard testing conventions with clear separation between unit tests and integration tests.

Important

#### Unit Tests

Unit tests must be added in the same file as the tested struct, enum, or trait. These tests should be placed in a
`tests` module at the end of the source file using the `#[cfg(test)]` attribute.

**For async tests** that require the tokio runtime, use the `#[tokio::test]` attribute and declare as `async fn`.
**For synchronous tests** that don't perform async operations, use the standard `#[test]` attribute with regular functions:

```rust
// src/my_module.rs
pub struct MyStruct {
    value: String,
}

impl MyStruct {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn do_something(&self) -> String {
        format!("Value: {}", self.value)
    }

    pub async fn do_something_async(&self) -> Result<String, std::io::Error> {
        // Async operation
        Ok(format!("Async value: {}", self.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synchronous test - use #[test]
    #[test]
    fn test_new() {
        let instance = MyStruct::new("test".to_string());
        assert_eq!(instance.value, "test");
    }

    // Synchronous test - use #[test]
    #[test]
    fn test_do_something() {
        let instance = MyStruct::new("test".to_string());
        let result = instance.do_something();
        assert_eq!(result, "Value: test");
    }

    // Async test - use #[tokio::test]
    #[tokio::test]
    async fn test_do_something_async() {
        let instance = MyStruct::new("test".to_string());
        let result = instance.do_something_async().await.unwrap();
        assert_eq!(result, "Async value: test");
    }
}
```

#### Integration Tests

Integration tests must be added in the `tests` folder at the same level as the `src` folder within each crate. Each
integration test file in the `tests` directory is compiled as a separate crate and can test the public API of your
crate:

```
my_crate/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    ├── integration_test.rs
    └── common/
        └── mod.rs
```

### Running Tests

Tests are organized in the `tests` directory of each crate. **We use `cargo nextest` exclusively for running tests** as
it provides faster test execution, better test organization, and improved test output compared to the standard cargo
test.

**All test execution must use the `cargo nextest run` command** with the appropriate flags:

#### Running Unit Tests

```
cargo nextest run -E 'kind(lib)' --all-features --workspace
```

#### Running Integration Tests

```
cargo nextest run -E 'kind(test)' --all-features --workspace
```

#### Additional Test Examples

```
# Run all tests
cargo nextest run --all-features --workspace

# Run a specific test
cargo nextest run --test my_integration_test

# Run tests with specific filter
cargo nextest run -E 'test(my_handler_test)'

# Run tests with partitioning (useful in CI)
cargo nextest run --partition count:1/3 --all-features

# Run tests in a specific package
cargo nextest run -p my_crate

# Run tests with verbose output
cargo nextest run --all-features --workspace --verbose
```

### Test Containers

The project uses the `testcontainers` crate to spin up Docker containers for integration tests. This is useful for testing against real databases, message brokers, or other external services.

### Tracing in Tests

Tests use the `tracing_test` crate to capture and assert on log messages:

```rust
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_message_handler() {
    // Test code...

    assert!(logs_contain("Expected log message"));
}
```

### Adding New Tests

When adding new tests:

1. Create a new test file in the `tests` directory of the appropriate crate
2. **Always use the `#[tokio::test]` attribute for all test functions** (both sync and async tests)
3. **Declare all test functions as `async fn`**
4. Use the `#[traced_test]` attribute if you need to assert on log messages
5. Make sure to enable the appropriate feature flags when running the test

Example of a simple test:

```rust
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn test_message_processing() {
    let message = TestMessage {
        content: "Hello, World!".to_string(),
    };

    let result = process_message(message).await;

    assert!(result.is_ok());
    assert!(logs_contain("Processing message"));
}
```
