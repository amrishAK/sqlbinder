# test-utils

This directory contains local test fixtures for the project’s PostgreSQL development environment.

It exists to keep the repository self-contained for testing schema, query mappings, and data-layer behavior without requiring a separate setup process.

## What’s inside

- `docker-compose.yaml` for starting PostgreSQL locally.
- `seed.sql` for creating and populating the test database.

## Why it exists

This project is built around explicit SQL, generated mappings, and local validation of schema compatibility, so a small fixture set makes development much easier.

The seed data includes:
- basic CRUD records,
- dataset metadata rows,
- join-friendly tables,
- enough variety to test both simple and more complex query paths.

This fixture is intentionally relational-only. Cache data is not part of this seed and should live in a separate store or fixture.

## Seed Tables

- `users` stores the base people records used by most examples and foreign-key relationships.
- `teams` groups users together and links back to a team owner.
- `team_members` models many-to-many membership between users and teams.
- `datasets` represents higher-level collections owned by teams.
- `dataset_items` stores item-level records inside a dataset and covers richer column types, nullable fields, and JSON metadata.
- `item_tags` adds extra labels per item so joins and filtering can be exercised.

## Quick start

1. Start PostgreSQL with Docker Compose on a fresh database volume.
2. The one-shot seed service will apply `seed.sql` after Postgres becomes healthy.
3. Point your local Rust config at the seeded database.

## Notes

- The seed data is for development and testing only.
- The schema is intentionally broad enough to support future query identifiers and validation cases.
- The seed is meant for CRUD, filtering, joins, and other relational query coverage.
- If you want to reseed from scratch, remove the Postgres volume with `docker compose down -v` and start the stack again.
- If you add new query examples later, update `seed.sql` first so the fixtures stay aligned.

## Suggested files

- `docker-compose.yaml`
- `seed.sql`
- `README.md`