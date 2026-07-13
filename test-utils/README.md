# test-utils

Local fixtures used by `sqlbinder` tests and development.

This directory contains:
- a local PostgreSQL stack plus seed data for relational query testing,
- filesystem fixtures used by unit tests in `src/utils`.

## Directory layout

- `postgres/docker-compose.yaml`: local PostgreSQL + one-shot seed service.
- `postgres/seed.sql`: schema, indexes, and seed rows.
- `unit-test-resources/settings/`: TOML fixtures for settings parsing and merge tests.
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

## Quick start (PostgreSQL)

Run from `test-utils/postgres`:

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

Rust tests in `src/utils/settings_handler.rs` and `src/utils/secret_handler.rs` load fixtures from:
- `test-utils/unit-test-resources/settings/`
- `test-utils/unit-test-resources/secrets/`

Settings fixture behavior covered by tests:
- `base_valid.toml`: valid single-file settings parsing.
- `base_without_environment.toml`: missing `environment` falls back to `development`.
- `invalid.toml`: TOML parse error path.
- `default_base.toml` + `default_development_override.toml`: merge of `settings.toml` with `settings.<environment>.toml`.

Secrets fixture behavior covered by tests:
- `db_password_with_newline.txt`: password file loading trims trailing newline/whitespace.

`src/utils/settings_handler.rs` also validates:
- missing `settings.toml` returns I/O error,
- missing `database.password_file` returns `MissingPasswordSource`,
- loaded settings are cached in-process via `OnceLock` by `get_settings()`.

These files are intended for automated tests. Keep fixture updates small and explicit so behavior changes stay reviewable.

## Scope and constraints

- Development/testing only. Not production infrastructure.
- Relational fixture only; non-relational/cache fixtures are out of scope here.
- Prefer a fresh Postgres volume when validating seed changes to avoid conflicts with existing local data.