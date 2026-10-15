# Features

Updated: 2026-10-15

sqlbinder is pre-v1. Public APIs and build policy are not yet finalized. This tracker records implementation state; release scope, acceptance, evidence, and progress are tracked in release documents, starting with [v1](../releases/v1.md). The [roadmap](roadmap.md) records high-level ideas. Things sqlbinder will not do are listed under [Non-goals](../project_overview.md#non-goals) in the project overview.

| Feature | State | Target | Notes / links |
|---|---|---|---|
| Settings and secret loading | Done | v1 | [Project overview](../project_overview.md#architecture) |
| PostgreSQL and SQLite connection pools | Done | v1 | [Project overview](../project_overview.md#architecture) |
| Raw SQL query execution | Done | v1 | [Query executor](../features/query-executor-feature-reference.md#current-implementation-raw-sql) |
| Backend-neutral query results | Done | v1 | [Project overview](../project_overview.md#architecture) |
| JSON result envelope | Done | v1 | [Project overview](../project_overview.md#product-capabilities-and-status) |
| SQL query contract validation and binding | In progress | v1 | [SQL query contract](../features/sql-query-contract-feature-reference.md); validation, preparation, and binding are implemented; executor integration is pending and tracked separately as [bound query execution](../features/query-executor-feature-reference.md#planned-bound-execution) |
| Bound query execution | Planned | v1 | [Query executor](../features/query-executor-feature-reference.md#planned-bound-execution) |
| Query execution limits (timeouts, result size) | Planned | v1 | [Query executor](../features/query-executor-feature-reference.md#6-limits); open question EX-1 |
| Database-enforced read-only execution | Planned | TBD | [Query executor](../features/query-executor-feature-reference.md#7-read-only-execution); open question EX-3 |
| Query manifest generation | Planned | v1 | [Query manifest proposal](../features/query-manifest-feature-reference.md) |
| Query classification | Planned | v1 | [Query manifest proposal](../features/query-manifest-feature-reference.md#query-classification) |
| Database schema introspection for manifest generation | Planned | v1 | [Schema introspection](../features/schema-introspection-feature-reference.md) |
| Project initialization and template generation | Planned | v1 | [Project initialization](../features/project-initialization-feature-reference.md) |
| CLI commands and project configuration loading | Planned | v1 | [CLI feature](../features/cli-feature-reference.md) |
| Embedded Rust project runtime (CLI tooling plus crate-based registered execution) | Planned | TBD | [Usage modes](../project_overview.md#two-usage-modes); existing library primitives do not provide the complete manifest-backed workflow. Public API, artifact loading, and lifecycle decisions remain open; no change to v1 scope. |
| Protobuf contract generation | Planned | TBD | [Query-to-Protobuf proposal](../features/query-to-protobuf-feature-reference.md) |
| Rust type generation for internal use | Planned | TBD | [Architecture](../project_overview.md#architecture); compiled versus dynamic runtime is undecided |
| REST middleware/API with JSON payloads (track 1), bearer authentication, request correlation logs, and published OpenAPI | Planned | v1 | [REST middleware](../features/rest-middleware-feature-reference.md) |
| REST API with protobuf payloads (track 2) | Planned | TBD | [Architecture](../project_overview.md#architecture) |
| gRPC API (track 3) | Planned | TBD | [Architecture](../project_overview.md#architecture) |
| Query discovery endpoint | Planned | v1 | [REST middleware](../features/rest-middleware-feature-reference.md) |
| Admin endpoints | Planned | TBD | [Responsibilities and limits](../project_overview.md#responsibilities-and-limits) |
| Static bearer token from server configuration | Planned | v1 | [REST middleware](../features/rest-middleware-feature-reference.md#3-authentication-and-caller-metadata); replaced by service-session authentication |
| Service-session token authentication and operation permissions | Planned | After v1 | [Responsibilities and limits](../project_overview.md#responsibilities-and-limits) |
| Extended observability: metrics, distributed tracing, and operational logging | Planned | TBD | Minimal v1 request logging is specified in the [REST middleware](../features/rest-middleware-feature-reference.md); broader observability is noted in [product capabilities](../project_overview.md#product-capabilities-and-status) |
| Audit events | Planned | TBD | [Product capabilities and status](../project_overview.md#product-capabilities-and-status) |
| Container base image and packaging | Planned | TBD | [How you would use it](../project_overview.md#how-you-would-use-it) |
| Pagination and streaming for large results | Planned | TBD | [Query executor](../features/query-executor-feature-reference.md#future); open question EX-2 |
| Multi-query transactions | Planned | TBD | [Query executor](../features/query-executor-feature-reference.md#future); open question EX-6 |
| Controlled write workflow | Planned | TBD | [Responsibilities and limits](../project_overview.md#responsibilities-and-limits) |
| Per-user authorization and trusted identity binding | Planned | TBD | [Responsibilities and limits](../project_overview.md#responsibilities-and-limits) |
