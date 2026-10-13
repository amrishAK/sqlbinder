# Project Overview

sqlbinder is a Rust library and sample executable for configuration-driven database access. The implementation loads settings, creates backend pools, executes SQL, normalizes result sets, and can format rows as JSON. It does not include an HTTP API layer, external query-definition runtime, or protobuf support.

## Goals

- support PostgreSQL and SQLite backends
- keep connection lifecycle isolated from query execution
- normalize query outputs across database engines
- serialize query results into a simple JSON-friendly envelope
- validate settings and secrets before creating a pool

## Non-goals

The repository does not currently provide:

- a REST service
- request routing or operation resolution
- an integrated external query-definition execution runtime
- response mapping layers beyond basic result formatting
- generated protobuf or API schemas

## Current runtime model

The library workflow is:

1. `ContextContainer::new()` loads `settings.toml` from the crate root and merges `settings.<environment>.toml` when that file exists.
2. The caller reads the configured `DatabaseSettings` and passes them to `get_connection_pool()`, which validates settings and constructs a PostgreSQL or SQLite pool.
3. The caller may store the pool in the context, whose pool slot can be initialized once, or pass the pool to `create_query_executor()`.
4. `QueryExecutor` executes SQL and returns normalized `QueryRows` for reads or an affected-row count for writes.
5. `query_result_formatter` can convert read rows to a JSON `count`/`items` envelope.

The sample executable follows the settings, pool, and context steps, then prints the environment name. It does not run SQL or serve requests.

## Key components

### `ContextContainer`

`ContextContainer` owns application settings and exposes the active environment and database configuration. It has a shared pool slot that callers can initialize once; constructing the context does not create a pool.

### `AppSettings`, `DatabaseSettings`, and `QuerySettings`

Settings are parsed into typed configuration structs. `AppSettings` includes
database settings and a required `queries.directory` path. The query directory
is configured but is not currently loaded by the runtime. The supported
database variants are:

- `DatabaseSettings::Postgres(PostgresSettings)`
- `DatabaseSettings::Sqlite(SqliteSettings)`
- `DatabaseSettings::SQLiteInMemory(SqliteInMemorySettings)`

The validator enforces required backend fields, positive connection limits and timeouts, and PostgreSQL password-file requirements. Settings are read from the crate root, independent of the launch directory. A relative password-file path is opened relative to the process working directory.

### `DbConnectionPool`

The connection layer resolves the configured backend and returns a unified pool enum. The actual connection types are kept backend-specific under `src/db/pg/` and `src/db/sqlite/`, while the public API presents a single dispatch point.

### `QueryExecutor`

`QueryExecutor` accepts a pooled database backend and exposes:

- `execute_write()` for DML/DDL statements that return affected rows
- `execute_read()` for SELECT-style queries that return `QueryRows`

The result model is backend-neutral and uses `DbValue` for row values.

### Query handler

`prepare_query()` loads a SQL file, validates named input parameters, and returns
backend-specific SQL with an ordered parameter vector in `BoundQuery`. The
executor does not yet consume this bound query. The handler validates statement
structure, input names, explicit output aliases, and wildcard restrictions;
the [SQL query contract feature](features/sql-query-contract-feature-reference.md)
records the rules and validation scope.

### `query_result_formatter`

The formatter converts `QueryRows` into a JSON object with a row count and items keyed by column name:

```json
{"count":1,"items":[{"id":1,"name":"alice"}]}
```

`count` is the number of rows. Values retain their JSON scalar form; byte values serialize as integer arrays. This is a generic formatter output, not a versioned service contract.

## Data flow

The effective data flow when a caller executes a read is:

1. read TOML settings
2. validate backend configuration
3. resolve password file for PostgreSQL when needed
4. create a backend pool
5. optionally store the pool in the shared context
6. build a `QueryExecutor` from the pool
7. run SQL and return `QueryRows` containing column names and typed values
8. optionally serialize rows to the JSON envelope

## Current status

The repository currently implements configuration, pooling, query execution, and JSON row formatting. The protobuf design in [the feature reference](features/query-to-protobuf-feature-reference.md) is planned work, not an implemented runtime path.