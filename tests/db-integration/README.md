# Database Integration Tests

These Cargo integration-test targets exercise the public `sqlbinder` API with
SQLite and PostgreSQL. Shared executor and seed helpers are in `support/`.

## Targets

- `sqlite/` runs with the default suite (`cargo test --test sqlite`). Each test
  creates its own one-connection in-memory database and applies the SQLite seed.
- `postgres/` runs a settings/executor-construction smoke test by default; it
  does not connect to PostgreSQL. Query tests are ignored unless requested
  explicitly and require the local PostgreSQL fixture.

Both targets split cases by behavior:

- `schema_seed.rs`: schema, table counts, and column names.
- `read_queries.rs`: reads, joins, filters, nullable values, and backend-specific
  metadata queries.
- `aggregation_queries.rs`: grouping, aggregates, `HAVING`, and numeric values.
- `mutation_queries.rs`: inserts, updates, deletes, and cascades.
- `query_errors.rs` (SQLite only): invalid SQL and database constraint errors.

The PostgreSQL target also covers JSONB queries. Its connection settings are
embedded from `settings/postgres.toml`; the configured password file is opened
relative to the process working directory. Run commands from the repository
root so that the fixture path resolves.

## Run PostgreSQL tests

Start the local database and seed service from the repository root:

```powershell
docker compose -f tests/test-utils/postgres/docker-compose.yaml up -d
```

Run the ignored PostgreSQL query tests:

```powershell
cargo test --test postgres -- --ignored --test-threads=1
```

The PostgreSQL tests share one database, and the helper reapplies the seed
before each test. Running serially avoids concurrent tests modifying that shared
fixture at the same time. Stop the services with:

```powershell
docker compose -f tests/test-utils/postgres/docker-compose.yaml down
```

The seed and database are local test fixtures, not production infrastructure.