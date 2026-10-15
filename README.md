# sqlbinder

sqlbinder is a Rust crate for teams that want to share database operations defined in reviewed SQL with explicit contracts across PostgreSQL and SQLite. Today it provides settings and secret loading, pooled database access, raw SQL execution, query-file validation and preparation, and typed result formatting; prepared-query execution and the API/CLI workflow are not implemented. It is a pre-v1 project aiming to support both an API service and embedded Rust usage.

## What it does

- loads app settings from a TOML file
- validates database configuration and secret-file inputs
- creates a pooled connection manager for PostgreSQL or SQLite
- validates SQL files against explicit input/output contracts and prepares backend-specific SQL with ordered parameters
- executes raw SQL through a backend-neutral `QueryExecutor`
- normalizes database rows into `DbValue` and `QueryRows`
- serializes results to a JSON-style `count`/`items` envelope

## Runtime

The core runtime is organized around a small set of primitives:

- `ContextContainer` loads settings and can hold a shared pool for the process
- `DatabaseSettings` models the configured backend and connection parameters
- `QuerySettings` requires a directory for SQL query files
- `get_connection_pool()` validates settings and selects the pool factory for the active backend
- `QueryExecutor` runs raw SQL strings against the selected pool and returns `QueryRows`
- `prepare_query()` reads and validates a SQL file, then returns backend SQL with ordered parameters; it does not execute the query
- `query_result_formatter` converts result rows into a JSON-compatible envelope

The configured query directory is not automatically loaded by the runtime, and the executor does not yet accept prepared queries. The executable loads settings, creates and stores a pool, then prints the environment name; it does not start a request-handling service.

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

[queries]
directory = "tests/test-utils/queries"
```

`queries.directory` is required and points to the directory reserved for SQL
query files. `prepare_query()` accepts a SQL file path directly; it does not
discover files from this setting or connect prepared queries to the executor.
The current `QueryExecutor` accepts raw SQL strings only, so callers are
responsible for ensuring raw SQL is safe.

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

Use Rust edition 2024 tooling and run commands from the repository root. A
minimum supported Rust version and supported platforms are not yet established.
CI and a shared pipeline entry point are not finalized; the current development
interface is direct Cargo commands.

Build the crate:

```bash
cargo build
```

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

The project currently covers configuration loading, pooled database access, raw SQL execution, SQL-file contract validation and preparation, row normalization, and JSON result formatting. Bound-query execution is not implemented; see the [query executor reference](docs/features/query-executor-feature-reference.md). The [query manifest](docs/features/query-manifest-feature-reference.md) and [protobuf generation](docs/features/query-to-protobuf-feature-reference.md) are proposals, not implemented features.

See the [feature tracker](docs/status/features.md) for the current state and target of each capability.
The [roadmap](docs/status/roadmap.md) records high-level direction; [v1 release acceptance](docs/releases/v1.md) tracks release scope, progress, and required functional, E2E, and benchmark evidence.

## Further documentation

- [Repository agent guide](AGENTS.md)
- [Project overview](docs/project_overview.md)
- [Current architecture and constraints](docs/architecture/architecture.md)
- [v1 release acceptance](docs/releases/v1.md)
- [SQL query contract](docs/features/sql-query-contract-feature-reference.md)
- [Query executor](docs/features/query-executor-feature-reference.md)
- [Query manifest proposal](docs/features/query-manifest-feature-reference.md)
- [Query-to-Protobuf proposal](docs/features/query-to-protobuf-feature-reference.md)
- [Test guide](tests/README.md)
- [Test fixtures](tests/test-utils/README.md)

## License

Licensed under the [Apache License 2.0](LICENSE).