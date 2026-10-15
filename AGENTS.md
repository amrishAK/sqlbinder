# Repository instructions

[sqlbinder](README.md) is a pre-v1 Rust crate for reviewed SQL contracts and
PostgreSQL/SQLite data access, with API-service and embedded Rust workflows
planned. It uses Rust edition 2024; a minimum supported Rust version and
supported-platform policy are not established.

## Repository and context map

| Path | Purpose and authority | Consult when |
|---|---|---|
| [README.md](README.md) | User-facing entry point, setup, examples, and current quick commands; keep it aligned with the implementation. | Orienting, using, or developing the crate. |
| [docs/project_overview.md](docs/project_overview.md) | Product vision, intended architecture, responsibilities, and non-goals; not a status or release commitment. | A change affects product direction, architecture, or boundaries. |
| [docs/architecture/architecture.md](docs/architecture/architecture.md) | Current components, lifecycle, public boundaries, and architecture constraints. | Changing production modules, errors, or public APIs. |
| [docs/status/features.md](docs/status/features.md) | Canonical implementation state and target for each capability. | Checking or changing feature status; update with the same change that changes the status. |
| [docs/status/roadmap.md](docs/status/roadmap.md) | High-level future direction only. | Checking product ideas; do not treat them as committed scope. |
| [docs/releases/v1.md](docs/releases/v1.md) | v1 scope, acceptance gates, progress, and required evidence. | Work affects v1 readiness; update affected gates with readiness changes. Passing requires linked candidate evidence. |
| [docs/features/](docs/features/), [SQL query contract](docs/features/sql-query-contract-feature-reference.md) | Feature requirements, SQL input/output rules, proposals, and open decisions. | Changing a feature or query contract; proposals are not current behavior. |
| [src/lib.rs](src/lib.rs), [src/context_container/](src/context_container/), [src/db/](src/db/), [src/query_result_formatter/](src/query_result_formatter/) | Implementation evidence: public exports, runtime context/settings, database abstractions/backends, and result formatting. Code determines what currently runs. | Verifying behavior, boundaries, and module ownership. |
| [Cargo.toml](Cargo.toml) | Rust edition, dependencies, and integration-test targets. | Checking the configured toolchain or test layout. |
| [tests/README.md](tests/README.md), [src/](src/) | Test-tier index and source-local unit tests, including [query-handler tests](src/db/query_handler/tests.rs). | Testing local contracts. |
| [tests/db-integration/README.md](tests/db-integration/README.md) | Public-API SQLite/PostgreSQL tests, shared helpers, fixture prerequisites, and serial execution guidance. | Selecting or provisioning database integration tests. |
| [tests/test-utils/README.md](tests/test-utils/README.md) | Ownership and purpose of test-only database, SQL, settings, and secret fixtures. | Editing or interpreting test fixtures. |

Read task-relevant linked authorities and any applicable path-scoped
instructions before changing an area. When sources disagree, distinguish implemented behavior (code and tests),
feature state, product intent, and release commitments using the authorities
above; report contradictions rather than silently promoting plans to behavior.

E2E tests and benchmark harnesses are not established. Their planned acceptance
requirements belong to [v1 release acceptance](docs/releases/v1.md), not an
existing runner or passing gate.

## Build and test

Run commands from the repository root.

| Task | Command |
|---|---|
| Build | `cargo build` |
| Default tests | `cargo test -q` |
| SQLite integration | `cargo test --test sqlite` |
| Start PostgreSQL fixture | `docker compose -f tests\test-utils\postgres\docker-compose.yaml up -d` |
| PostgreSQL query tests | `cargo test --test postgres -- --ignored --test-threads=1` |
| Stop PostgreSQL fixture | `docker compose -f tests\test-utils\postgres\docker-compose.yaml down` |

- Default tests must not require provisioning an external service.
- PostgreSQL query tests share mutable fixture state; run serially and follow the [integration test guide](tests/db-integration/README.md).
- CI, formatting, linting, coverage, and a shared pipeline entry point are not finalized. Do not claim these gates exist or add tooling without approval.

## Documentation maintenance

- Keep current behavior separate from planned work. Proposals belong under `docs/features/` and must state implementation status; unresolved decisions remain open questions.
- Keep [docs/status/roadmap.md](docs/status/roadmap.md) high-level. Put detailed release scope, gates, progress, and evidence in `docs/releases/`.
- Keep [docs/project_overview.md](docs/project_overview.md) concise and about product vision, intended capabilities and usage, architecture/data flow, responsibilities, and non-goals. Do not put implementation-status tables, progress, release acceptance, or open questions there; link mutable state and feature detail instead. Update it only when product vision, boundaries, or architecture change.
- Keep every feature reference in a shared shape: title, status badge, `Summary`, `Scope` with in-scope and out-of-scope links, numbered `###` body sections, and `Open questions` with an `Answered` subsection where applicable.
- Update [docs/status/features.md](docs/status/features.md) in the same change that starts, finishes, or drops a feature. Correctness, performance, and coverage claims require linked evidence.

## Commits and agent boundaries

- Follow Conventional Commits using only `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, or `chore`.
- Do not edit generated Cargo output under `target/`.
- Never expose credentials or secret contents. Treat credentials under `tests/` as test-only; never reuse or present them as production secrets.
- Do not publish crates, create releases, change repository secrets, or introduce CI/build infrastructure without explicit approval.
- Preserve unrelated working-tree changes, especially ongoing feature documentation edits.
