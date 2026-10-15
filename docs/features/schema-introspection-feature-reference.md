# Database Schema Introspection Feature

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

Manifest generation needs database schema information to resolve query input
and output types and nullability. The planned generator connects to the
project's configured PostgreSQL or SQLite database and inspects the selected
schema directly. Users do not need to create a schema snapshot for the normal
workflow.

No schema introspection behavior is implemented yet.

## Scope

In scope:

- selecting the configured database and schema/catalog to inspect
- retrieving metadata needed by manifest type and nullability resolution
- reporting unsupported or ambiguous metadata as explicit errors
- using the configured file-based database credential for PostgreSQL
- keeping introspection separate from query execution and manifest writing

Out of scope for this document:

- initializing project files, covered by the [project initialization feature](project-initialization-feature-reference.md)
- CLI command dispatch, covered by the [CLI feature](cli-feature-reference.md)
- query classification and manifest format, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- bound query execution, covered by the [query executor feature](query-executor-feature-reference.md)
- database migrations and provisioning, which are project/operator responsibilities

## 1. Introspection workflow

`generate-manifest` loads the selected project's settings, opens a connection
to the configured database, inspects the configured schema/catalog, and passes
the required metadata to manifest generation. The command must not modify
database schema or data. Schema inspection must not execute project queries to
obtain sample values.

The generator must state which database and schema/catalog it inspected. If
the configuration does not identify a unique target, generation fails with an
actionable error rather than silently choosing a schema.

## 2. Credential source

The initial credential provider is a password file configured by
`database.password_file`. The initialized project uses an ignored local
settings overlay that points to `db.local.pwd`; `init` creates the file empty
and never generates a password. Users provide the file locally or mount a
secret at the configured path in a container.

The password file must not be included in the manifest, printed in diagnostics,
or passed as a command-line password argument. Missing, unreadable, or empty
credentials fail explicitly. The same configured provider may be used for
manifest generation and server startup; separate build-time and runtime
credentials have not been decided.

The configuration should keep credential selection behind a narrow setting
boundary so other providers can be considered later. Key vault and IAM
integration are not part of the initial file-based scope and are not promised
by this proposal.

## 3. Backend and metadata contract

PostgreSQL and SQLite need backend-specific metadata inspection behind the
shared database abstractions. The implementation must document the minimum
permissions needed and avoid requiring broader database privileges than the
metadata and query contract analysis actually need.

Type resolution uses metadata, not sample result values. If a query expression
or parameter cannot be resolved from the inspected schema and the supported
SQL analysis, generation returns an explicit unresolved-contract error; it
does not guess. The supported database type mappings are owned by the
[query manifest proposal](query-manifest-feature-reference.md).

## 4. Open questions

| ID | Question | Needed by |
|---|---|---|
| SI-1 | How are PostgreSQL schemas and SQLite databases/catalogs selected, and what happens when selection is ambiguous? | Introspection implementation |
| SI-2 | Which catalog metadata is required per backend, and what are the minimum permissions to read it? | Introspection implementation |
| SI-3 | Can query types and nullability be resolved from metadata and SQL analysis for the supported query set, especially expressions and CTEs? | Manifest implementation |
| SI-4 | Should an optional schema export/import format be supported for offline or reproducible generation? | Future workflow |
| SI-5 | Should generation and runtime support separate credential files or other credential providers in the initial release? | Credential contract |
| SI-6 | Can SQLite in-memory databases be used for manifest generation, given that their schema exists only for the lifetime of a connection? | Backend support |

### Answered

None yet.
