# Project Overview

**sqlbinder is being built to make reviewed SQL reusable as a clear, explicit
data-access interface.** A team will define database operations in SQL,
generate a contract for each operation, and let applications call those
registered operations instead of each building and maintaining its own query
wrapper.

The product vision has two usage modes:

- **API service:** run sqlbinder separately and let applications call
  registered operations through an API.
- **Rust crate:** use sqlbinder from a Rust application to execute those
  operations in-process, without running the API service.

sqlbinder is intended to handle the database-access boundary, not the whole
backend. Applications keep their business logic, end-user identity, and
authorization.

> **Product vision:** This document describes the product sqlbinder is being
> built to become; it is not a statement of current availability or a release
> commitment. See the [feature tracker](status/features.md) for what is
> implemented and the [roadmap](status/roadmap.md) for the high-level product
> direction.

## The problem

When several applications need the same database operations, teams can end
up duplicating SQL integration, parameter handling, and result mapping.
Building a custom API centralizes access, but its handlers and contracts must
also be written and kept aligned with the queries.

sqlbinder's proposed middle ground is to keep SQL visible and reviewed while
making each operation's inputs and returned fields explicit. The aim is to
reduce repeated data-access plumbing and provide a shared interface without
hiding SQL behind an ORM or moving business workflows into the database layer.

## What sqlbinder is designed to do

The vision is to let a team:

- keep database operations as reviewable SQL in its project;
- validate queries and define explicit input and output contracts;
- generate a manifest that describes registered operations and their
  contracts;
- expose those operations through an API service or execute them through a
  Rust crate; and
- keep database connections and execution behind the chosen data-access
  boundary.

A **registered operation** is a reviewed SQL statement defined in the
project's SQL files, identified in the generated manifest, and described by
its accepted inputs and declared outputs. At runtime, callers invoke that
operation and provide values; they do not submit SQL.

These are product goals, not measured productivity, compatibility, or
performance claims.

## How you would use it

The intended end-to-end workflow is:

1. **Set up the project.** Use the CLI to create project configuration and a
   directory for SQL operations. Keep credentials out of source control.
2. **Write and review SQL.** Define a supported statement for each operation,
   use named `$name` inputs, and give returned fields explicit, unique aliases.
   Wildcard projections are not part of the query contract.
3. **Validate and generate contracts.** The CLI validates the SQL and
   inspects the configured database schema to resolve types. It generates a
   manifest describing operation identities, kinds, inputs, and outputs.
4. **Choose a runtime mode.** Run the API service with the project settings
   and manifest, or load the project artifacts from a Rust application using
   the sqlbinder crate.
5. **Call an operation.** Identify the registered operation and provide its
   input values. The runtime validates and binds the inputs, executes the
   operation, and returns its declared result or an explicit error.

This is the intended product experience, not a guide to available commands.
The [feature tracker](status/features.md) records current implementation
state.

### Example: customer lookup

Suppose an application needs to retrieve a customer by ID. The team keeps the
operation in SQL:

```sql
SELECT id AS customer_id, name AS customer_name
FROM customers
WHERE id = $customer_id;
```

In the intended workflow, tooling validates the query and records its
`customer_id` input and `customer_id` and `customer_name` outputs in the
manifest. An API caller would invoke the registered operation with a
`customer_id` value; a Rust application would invoke the same operation
through the crate. Neither caller sends SQL. Adding another column to the
table would not automatically add it to this operation's result.

This is an illustrative product example, not a runnable API request. The
application must still decide whether the current user is allowed to view
that customer.

## Two usage modes

| Mode | How an application uses it | Intended fit |
|---|---|---|
| API service | Call registered operations through an API. The interface direction includes REST with JSON, REST with Protocol Buffers, and gRPC. | Applications in different languages or teams that want a shared service boundary. |
| Rust crate | Use project tooling to validate SQL and generate a manifest, then load it and execute registered operations in-process through the crate. | Rust applications that want to own their HTTP and business layers without a separate sqlbinder service. |

Both modes are intended to expose the same registered operations and honor
the same input and output contracts. The CLI is a project-time tool, not an
additional runtime service for embedded applications. The API service adds
a process and a network hop; crate consumption keeps execution in the Rust
application's process and couples upgrades to that application. The
surrounding application retains end-user authentication and authorization
in either mode.

## Is sqlbinder for you?

sqlbinder may be a fit if your team prefers SQL, has a defined set of database
operations, and wants to make those operations reusable across applications
or services. It is especially relevant when maintaining separate query
wrappers for each consumer is becoming repetitive.

Potential uses include:

- sharing database operations across applications written in different
  languages;
- providing a data-access layer for internal tools or read-oriented APIs;
- using the Rust crate inside an application that owns its business logic and
  HTTP interface; and
- separating an application's read operations from its business workflows.

It may not be a fit if:

- one application already has a suitable data-access layer and an additional
  service or library would add more work than it removes;
- consumers need arbitrary SQL, ad hoc analytics, or unrestricted query
  composition;
- the data-access layer must coordinate complex business workflows or
  multi-query transactions; or
- you need a complete backend with built-in end-user identity, tenant
  isolation, and per-user authorization.

For web and mobile applications, the intended boundary is:

```text
Client -> trusted application backend -> sqlbinder service or Rust crate -> database
```

Do not put database credentials or a shared service token in a browser or
mobile client. sqlbinder is not end-user authentication, and neither usage
mode removes the need for a trusted application backend.

## Architecture

The intended design separates project-time contract generation from runtime
execution:

```text
PROJECT TIME
SQL files + database schema
          |
          v
CLI validates SQL and generates a manifest
          |
          v
Versioned project artifacts: SQL + manifest + configuration
          |
          +--------------------------------------+
          |                                      |
          v                                      v
RUNTIME: API service                    RUNTIME: Rust application
REST/JSON                               sqlbinder crate (in-process)
REST/Protocol Buffers and gRPC          no sqlbinder server
          |                                      |
          +------------------+-------------------+
                             v
                   Shared execution design
             validate inputs -> bind parameters
                 -> execute -> shape results
                             |
                             v
                  PostgreSQL or SQLite
```

The CLI is intended to validate SQL, inspect schema, and generate the
manifest before deployment. At runtime, the service or Rust crate loads those
artifacts; it does not regenerate the manifest for each call. The API
transport and service authentication belong to service mode. In crate mode,
the Rust application owns its process lifecycle and request handling.

The main architectural parts are:

| Part | Responsibility in the vision |
|---|---|
| CLI and contract tooling | Initialize project files, validate and classify SQL, inspect the schema, and generate the manifest. |
| Manifest | Describe operation identity, kind, input types, and declared output fields for runtime use. |
| API service | Authenticate service callers, expose operation discovery and execution, and return results over the selected API interface. |
| Rust crate | Load project artifacts and provide registered-operation execution inside a Rust application without HTTP or server startup. |
| Query execution | Bind validated inputs, execute against the selected database backend, and return contract-shaped results with explicit errors and limits. |
| Database backends | Provide the PostgreSQL and SQLite connection and query implementations behind shared pool and execution abstractions. |

REST with JSON is the initial API direction; REST with Protocol Buffers and
gRPC are additional interface directions. The embedded Rust workflow,
container packaging, and generated language-specific types are also part of
the broader product direction. The [roadmap](status/roadmap.md) records
high-level direction, while the [feature tracker](status/features.md) records
implementation state.

## Responsibilities and limits

sqlbinder is intended to validate operation contracts, bind inputs, execute
registered SQL, and return declared results. The surrounding application
remains responsible for:

- business rules and workflows;
- end-user identity and authorization decisions;
- deciding which callers may invoke which operations; and
- the application's request lifecycle and user-facing API.

Operators remain responsible for secret storage, network isolation, gateway
controls such as rate limiting, appropriate database privileges, and
protecting operational data. Explicit contracts can reduce accidental data
exposure; they do not by themselves make an operation authorized or secure.
A shared bearer token does not identify individual users.

sqlbinder is not intended to be:

- a full backend framework or a place for business logic;
- an end-user identity or account-management platform;
- a service that executes client-supplied SQL;
- a database migration tool;
- a workflow coordinator or general-purpose transaction manager; or
- a guarantee of infrastructure security, network isolation, or secret-store
  protection.

Reads are the recommended adoption starting point, not a promise of enforced
read-only execution. Registered write operations are part of the intended
product; registration alone does not make a write safe or authorized. Use
database privileges appropriate to the operations and confirm available
controls before relying on write functionality.

The intended product is a small database-access boundary built around
reviewed SQL and explicit operation contracts, available as either a service
or a Rust crate while leaving business workflows and end-user authorization
to the surrounding application. This overview describes that product
direction; the [feature tracker](status/features.md) records implementation
state, and the [roadmap](status/roadmap.md) records high-level priorities.
