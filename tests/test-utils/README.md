# Test Fixtures

Local database and filesystem fixtures used by the tests. Integration test
targets and commands are documented in [`../db-integration/README.md`](../db-integration/README.md).

This directory contains:
- a local PostgreSQL stack plus seed data for relational query testing,
- a SQLite-compatible seed schema used by external integration tests,
- settings and secret-file fixtures used by unit tests in `src/context_container/app_settings/`.

## Directory layout

- `../db-integration/`: external SQLite and PostgreSQL integration tests and shared helpers.
- `postgres/docker-compose.yaml`: local PostgreSQL + one-shot seed service.
- `postgres/seed.sql`: schema, indexes, and seed rows.
- `sqlite/seed.sql`: SQLite-compatible schema, indexes, and equivalent seed rows.
- `queries/`: parameterized SQL query fixtures shared by tests.
- `unit-test-resources/settings/`: TOML fixtures for settings parsing, defaults, validation, and merge tests.
- `unit-test-resources/secrets/`: secret-file fixtures for password file loading tests.

## PostgreSQL fixture

The compose stack defines:
- `postgres` (`postgres:16-alpine`) with:
	- database `appdb`,
	- user `appuser`,
	- password `apppass`,
	- host port `5432` mapped to container `5432`.
- `seed` (`postgres:16-alpine`) that waits for a healthy database and runs:
	- `psql -h postgres -U appuser -d appdb -v ON_ERROR_STOP=1 -f /seed.sql`

Seeded tables:
- `users`
- `teams`
- `team_members`
- `datasets`
- `dataset_items`
- `item_tags`

The seed script also creates indexes and inserts sample rows covering CRUD, joins, filtering, nullable fields, and JSONB metadata queries.

## SQLite fixture

The SQLite seed is loaded into a fresh in-memory database by `tests/db-integration/support/database.rs`
for every test in the `sqlite` target.
It covers the same relational entities and sample records as the PostgreSQL fixture,
using SQLite-compatible types. JSON metadata is stored as `TEXT` because SQLite does
not provide PostgreSQL's `JSONB` type. The external tests cover table counts, joins,
filters, null values, metadata matching, aggregation, numeric scores, tag lookups,
updates, uniqueness errors, and foreign-key cascades.

## Quick start (PostgreSQL)

Run from `tests/test-utils/postgres`:

```bash
docker compose up -d
```

Check service status:

```bash
docker compose ps
```

Optional: view seed logs:

```bash
docker compose logs seed
```

Stop services:

```bash
docker compose down
```

Reset to a fresh database and reseed:

```bash
docker compose down -v
docker compose up -d
```

## Unit-test filesystem fixtures

Rust unit tests in `src/context_container/app_settings/` load fixtures from:
- `tests/test-utils/unit-test-resources/settings/`
- `tests/test-utils/unit-test-resources/secrets/`

Settings fixture behavior covered by tests:
- `base_valid.toml`: valid single-file settings parsing.
- `base_without_environment.toml`: missing `environment` falls back to `local`.
- `invalid.toml`: TOML parse error path.
- `default_base.toml` + `default_development_override.toml`: recursive merge of a base TOML value with an environment overlay.

Secrets fixture behavior covered by tests:
- `db_password_with_newline.txt`: password file loading trims trailing newline/whitespace.

The settings and secret handler unit tests use these fixtures to check parsing,
environment defaults and overlays, invalid TOML, and password-file loading.

These files are intended for automated tests. Keep fixture updates small and explicit so behavior changes stay reviewable.
Database setup for integration tests is implemented in
`tests/db-integration/support/database.rs`; seed SQL belongs here under the
matching backend directory. Settings are loaded from the crate root, but
relative password-file paths resolve from the process working directory.

The valid `queries/` fixtures use named `$name` inputs and explicit `AS` output
aliases, targeting tables created by both backend seed scripts.
`invalid_output.sql` intentionally omits an output alias to test contract
rejection before input-value validation. The query handler translates canonical
inputs to backend placeholders and returns ordered values; the current
`QueryExecutor` API does not bind these parameters.

## Scope and constraints

- Development/testing only. Not production infrastructure.
- Relational fixture only; non-relational/cache fixtures are out of scope here.
- Prefer a fresh Postgres volume when validating seed changes to avoid conflicts with existing local data.