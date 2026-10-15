# Current Architecture

This document owns the current implementation boundaries and architecture
constraints for sqlbinder. It is not a description of the complete planned
product. Read the [project overview](../project_overview.md) for product vision,
responsibilities, and non-goals; the [feature tracker](../status/features.md)
for implementation state; and [v1 release acceptance](../releases/v1.md) for
release scope and required evidence.

## Components and responsibilities

| Component | Current responsibility |
|---|---|
| [Public crate entry point](../../src/lib.rs) | Exports the context, database, and result-formatter modules. Preserve the existing module organization and public re-exports here and in module `mod.rs` files. |
| [Context and settings](../../src/context_container/) | Loads and validates configuration and secret-file inputs; holds settings and an explicitly initialized shared pool. |
| [Connection pools](../../src/db/connection_pool/) | Validates database settings and selects the backend-specific pool through a shared abstraction. |
| [PostgreSQL](../../src/db/pg/) and [SQLite](../../src/db/sqlite/) | Own backend-specific pool, connection, query-execution, and row-conversion details. |
| [Query preparation](../../src/db/query_handler/) | Loads and parses SQL files, validates contracts and supplied inputs, and returns backend SQL with ordered parameter values. |
| [Query execution](../../src/db/query/) | Dispatches raw SQL reads and writes to the selected backend, exposing backend-neutral results. |
| [Result formatting](../../src/query_result_formatter/) | Converts query rows into a JSON-compatible `count`/`items` envelope. |

Keep PostgreSQL and SQLite details in their backend modules and expose them
through the shared pool and query abstractions in [src/db/](../../src/db/).
Keep settings loading and process context in
[src/context_container/](../../src/context_container/).

## Lifecycle and data flow

[ContextContainer::new()](../../src/context_container/context_container_handler.rs)
loads settings and creates an empty shared pool slot. Constructing a context
must not implicitly create a database pool. Pool creation is a separate call
to the [pool factory](../../src/db/connection_pool/factory.rs); the caller can
then store the pool in the context.

The [executable](../../src/main.rs) loads settings, creates and stores a pool,
and prints the environment name. It does not start an API service or provide
the planned CLI workflow.

The current query paths are separate:

```text
SQL file + named input values + backend
  -> parse -> validate contract and inputs -> bind
  -> BoundQuery (backend SQL + ordered values; no execution)

Database pool + caller-controlled raw SQL
  -> QueryExecutor -> backend execution
  -> QueryRows for reads / affected-row count for writes
```

[prepare_query()](../../src/db/query_handler/handler.rs) returns
`BoundQuery` but does not execute it.
[QueryExecutor](../../src/db/query/executor.rs) currently accepts raw SQL
only; its caller is responsible for SQL safety. Do not make it consume
`BoundQuery` or connect preparation and execution as incidental work.
Bound execution is planned in the
[query executor reference](../features/query-executor-feature-reference.md),
with unresolved design questions; these constraints do not authorize its
implementation or resolve those questions.

## Contracts and failure boundaries

- Preserve the [SQL query contract](../features/sql-query-contract-feature-reference.md):
  one supported statement, canonical `$name` inputs, explicit unique final
  output aliases, no wildcard projections, and explicit errors for invalid
  contracts. The reference owns the detailed rules and exceptions.
- Keep module-specific typed errors and explicitly map lower-level errors at
  boundaries. Do not introduce silent fallback behavior.
- Never expose credentials or secret contents. Never expose raw SQL execution
  to API callers.
- Follow the [product non-goals](../project_overview.md#responsibilities-and-limits):
  do not add a full backend framework, business logic, client-supplied SQL
  execution, database migrations, identity management, or infrastructure-security
  guarantees. The planned service is a data-access boundary, not a full backend.

## Tests and implementation evidence

Unit tests remain source-local. Test fixtures and test infrastructure must stay
outside [src/](../../src/). See the [test guide](../../tests/README.md),
[database integration guide](../../tests/db-integration/README.md), and
[fixture guide](../../tests/test-utils/README.md) for tier ownership and commands.

[Query-handler tests](../../src/db/query_handler/tests.rs) exercise preparation
and binding; [executor unit tests](../../src/db/query/executor.rs) and
[database integration tests](../../tests/db-integration/) exercise raw SQL
execution. These are implementation evidence, not acceptance evidence for the
complete planned registered-operation workflow.
