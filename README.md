# sqlbinder

sqlbinder is a Rust crate for loading application configuration, creating backend-specific database pools, and executing SQL against PostgreSQL or SQLite. The project is intentionally focused on the execution layer and typed result handling rather than a full HTTP API service.

## What it does

- loads app settings from a TOML file
- validates database configuration and secret-file inputs
- creates a pooled connection manager for PostgreSQL or SQLite
- executes raw SQL through a backend-neutral `QueryExecutor`
- normalizes database rows into `DbValue` and `QueryRows`
- serializes results to a JSON-style `count`/`items` envelope

## Runtime

The core runtime is organized around a small set of primitives:

- `ContextContainer` loads settings and can hold a shared pool for the process
- `DatabaseSettings` models the configured backend and connection parameters
- `get_connection_pool()` validates settings and selects the pool factory for the active backend
- `QueryExecutor` runs SQL against the selected pool and returns `QueryRows`
- `query_result_formatter` converts result rows into a JSON-compatible envelope

The executable currently loads settings, creates and stores a pool, then prints the environment name. It does not start a request-handling service.

## Repository layout

- `src/context_container/` – app settings, environment metadata, and shared runtime context
- `src/db/` – pool dispatch, PostgreSQL, SQLite, and query execution
- `src/query_result_formatter/` – JSON envelope conversion for query results
- `tests/` – unit tests and database integration coverage

## Configuration

The loader reads `settings.toml` from the crate root and optionally deep-merges `settings.<environment>.toml` from that same directory. Relative `password_file` paths are opened relative to the process working directory, so run the sample commands from the repository root unless the path is absolute.

```toml
environment = "development"
database_type = "postgres"

[database]
host = "localhost"
port = 5432
user = "appuser"
name = "appdb"
password_file = "tests/test-utils/unit-test-resources/secrets/db_password_with_newline.txt"
max_connections = 10
connect_timeout_secs = 5
ssl_mode = "disable"
```

Supported backends:

- `postgres`
- `sqlite`
- `in-memory` (SQLite in-memory mode; `memory` and `sqlite-in-memory` are also accepted)

The settings validator checks required backend fields and connection limits. PostgreSQL requires a readable, non-empty password file. The example file uses a test fixture password and must not be used as a production secret.

## Example usage

```rust
use sqlbinder::context_container::ContextContainer;
use sqlbinder::db::{get_connection_pool, QueryExecutor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = ContextContainer::new()?;
    let pool = get_connection_pool(context.get_database_settings().as_ref())?;
    let executor = QueryExecutor::new(pool);

    executor
        .execute_write("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
        .await?;

    let rows = executor
        .execute_read("SELECT id, name FROM users ORDER BY id")
        .await?;

    println!("{rows:#?}");
    Ok(())
}
```

## Development and testing

Run the standard test suite:

```bash
cargo test -q
```

The SQLite integration target uses a fresh in-memory database per test. PostgreSQL query tests are ignored by default and require the local fixture stack; see [the integration test guide](tests/db-integration/README.md).

Run the SQLite integration target:

```bash
cargo test --test sqlite
```

To run the PostgreSQL query tests after starting the fixture stack, use the command in that guide.

## Status

The project currently covers configuration loading, pooled database access, SQL execution, row normalization, and JSON result formatting. Protobuf generation, external SQL definition loading, and request/response mapping are not implemented; the [protobuf feature reference](docs/features/query-to-protobuf-feature-reference.md) is a proposal.

## Further documentation

- [Project overview](docs/project_overview.md)
- [Test guide](tests/README.md)
- [Test fixtures](tests/test-utils/README.md)