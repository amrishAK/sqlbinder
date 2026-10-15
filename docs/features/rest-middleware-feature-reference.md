# REST Middleware Feature

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

The REST middleware exposes registered SQL operations without exposing SQL
execution to callers. Clients can list available operations, inspect one
operation's typed contract, and execute it with JSON input values. The service
uses the generated manifest and bound query executor.

This feature is planned and not implemented. V1 requires a published OpenAPI
document. Endpoint paths, HTTP methods, wire schemas, status codes, and health
behavior will be defined during implementation and documented in that
specification.

## Scope

In scope:

- listing registered operations
- retrieving one operation's public name/identity, kind, input schema, and output schema
- executing one registered operation with typed JSON inputs
- authenticating every endpoint with a static bearer token for v1
- writing a structured request log with operation and outcome/status
- optionally correlating requests using caller-supplied `user_id` and `span_id` headers
- publishing an OpenAPI document for the implemented API
- returning contract-defined JSON results and explicit errors
- ensuring callers cannot submit SQL

Out of scope for this document:

- project setup and CLI lifecycle, covered by the [CLI feature](cli-feature-reference.md)
- project scaffold contents, covered by the [project initialization feature](project-initialization-feature-reference.md)
- schema inspection, covered by the [schema introspection feature](schema-introspection-feature-reference.md)
- operation classification and contract storage, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- bound execution and result mapping, covered by the [query executor feature](query-executor-feature-reference.md)
- protobuf and gRPC transports, covered by the [project overview](../project_overview.md#architecture)
- per-user identity and authorization, deferred beyond the v1 static-token model

## 1. Operation terminology

An API **operation** is one registered query entry in the generated manifest.
Its stable manifest `id` is the execution and lookup identity. The word
operation does not mean a client may submit arbitrary SQL or choose a database
statement.

The public detail contract is derived from the manifest and contains, at
minimum:

- the stable operation `id`
- a display name only if the manifest contract defines one
- its classified kind (`read`, `insert`, `update`, `delete`, or `upsert`)
- its input field names, types, and nullability
- its output field names, types, and nullability when it returns rows
- its result kind: rows or affected-row count

Prepared SQL, raw SQL sources, parameter values, credentials, and internal
database configuration are not exposed by discovery.

## 2. Endpoint capabilities

The API provides three logical capabilities. These are resource-level
capabilities; their concrete paths and methods will be finalized during
implementation and described in the published OpenAPI document.

| Capability | Request | Response |
|---|---|---|
| List operations | No SQL input; optional filters are not yet specified | Registered operation identities and any agreed summary fields |
| Get operation details | One registered operation `id` | Public operation contract, including typed inputs and outputs |
| Execute operation | One operation `id` and a JSON object of input values | Declared row result or affected-row result |

The execute request must identify exactly one registered operation. Values are
bound as data and never interpolated into SQL. Missing, extra, null, or
wrong-type values are validated against the operation contract and produce
explicit errors without executing invalid requests.

## 3. Authentication and caller metadata

Every v1 endpoint, including listing and operation details, requires the
configured static token in the HTTP `Authorization` header using bearer-token
semantics.

The token is server configuration, not part of the project manifest, query
inputs, or response. Missing, malformed, or incorrect tokens are rejected
before returning operation metadata or executing database work. The v1 token
does not identify an individual user or grant per-operation permissions; all
authorized callers share its access.

Requests may optionally include `user_id` and `span_id` headers. When present,
the service records their values as caller-supplied correlation metadata in
structured request logs, alongside the operation identity when known and the
request/operation outcome and HTTP status. These values help an operator find
related log entries; they are not authenticated identity, authorization
claims, or proof that a particular user performed an operation. Do not use
them to grant permissions or make access-control decisions.

Treat both header values as untrusted input: constrain their length and
character set, encode them as structured log fields rather than concatenating
them into log lines, and avoid recording values that contain secrets or
sensitive personal data. Never log the bearer token, password, raw SQL, or
query parameter values. If a future service-session token supplies a trusted
identity, keep that identity distinct from these caller-provided correlation
fields.

The exact HTTP header spelling/casing and validation limits remain open.
`span_id` here is a caller correlation value, not necessarily a standards-based
distributed tracing span; compatibility with standard trace-context headers
is undecided.

## 4. Results and errors

Execution returns JSON consistent with the declared manifest result:

- row-producing operations return the declared output fields, including an
  empty row set with its output contract intact
- writes without `RETURNING` return an affected-row result
- decoding or contract mismatches are errors, not null substitutions

The exact JSON envelope and error body/status mapping remain to be specified.
Errors must distinguish at least authentication failure, unknown operation,
invalid input, and database/execution failure without exposing credentials,
raw SQL, or sensitive database details. Failed and rejected requests must not
be represented as successful empty results.

Execution inherits the timeout, result-size, and connection bounds defined by
the [query executor feature](query-executor-feature-reference.md#6-limits).
Pagination and streaming are not part of the initial API contract.

## 5. OpenAPI contract

V1 publishes an OpenAPI document as the discoverable API contract. It must
describe every implemented public route, method, request and response schema,
authentication requirement, status/error response, and health/readiness
endpoint. Its schemas and behavior must match the running implementation.
Changes to the API are not complete until the document is updated.

Routes, methods, exact JSON schemas and errors, server-side token
configuration, health/readiness behavior, and operation display names are
implementation decisions. Record the settled choices in the OpenAPI document
and keep any relevant explanations here. The remaining publication decision
is how the document is distributed (for example, a checked-in artifact, a
document served by the API, or both).

## 6. Open questions

| ID | Question | Needed by |
|---|---|---|
| REST-1 | Where and how is the OpenAPI document published, and how is its version kept aligned with the API implementation? | API implementation |
| REST-2 | Does the operation list return only IDs or also summary metadata, and are filters or pagination needed for the expected manifest size? | API contract |
| REST-3 | What exact header spelling and validation limits apply to optional `user_id` and `span_id`, and should standard trace-context headers also be accepted? | Middleware implementation |
| REST-4 | What readiness/liveness behavior, bind-address configuration, and graceful shutdown contract does the server expose, and which health endpoints (if any) are unauthenticated? | Server integration |
| REST-5 | Is a stable operation `id` sufficient as the public name, or does the manifest need display names/descriptions? | API contract |

### Answered

- The v1 authentication scheme is a static token in `Authorization: Bearer`.
- All v1 endpoints require authentication; the shared token does not represent an individual user.
- The API exposes list, detail, and execute capabilities for registered operations only; it never accepts caller-supplied SQL.
- Optional caller-supplied `user_id` and `span_id` headers are logged as untrusted correlation metadata; neither is trusted identity or authorization input.
- Minimal structured request logging is in v1: operation identity when known, outcome/status, and the optional correlation-header values. This is not the broader future observability/audit scope.
- V1 publishes an OpenAPI document. Exact routes, methods, JSON schemas, errors, token configuration, health/readiness behavior, and display names are defined during implementation and captured in that document.
