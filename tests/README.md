# Tests

The repository has unit tests embedded in `src/` and external database
integration tests under `tests/db-integration/`.

## Run the suite

Run the default suite:

```powershell
cargo test
```

This runs the source-level unit tests, the SQLite integration target, and a
PostgreSQL settings/executor-construction smoke test that does not connect to
the database. PostgreSQL query tests are ignored by default because they
require a local PostgreSQL database.

Run a specific integration target or module:

```powershell
cargo test --test sqlite
cargo test --test sqlite aggregation_queries
```

For PostgreSQL setup and the command to run its ignored tests, see
[`db-integration/README.md`](db-integration/README.md). Fixture ownership and
reset instructions are in [`test-utils/README.md`](test-utils/README.md).
