# Project Overview

sqlbinder is a deployable SQL execution service and not a reusable library. The intended deployment model is a containerized runtime that starts with user-provided application settings, SQL query definitions, and request/response mapping definitions.

## Service Requirement

The system shall expose REST endpoints and execute database queries in response to incoming HTTP requests.

At startup, the service shall:

1. Load application configuration.
2. Load query definitions from configured files.
3. Load request and response mapping definitions.
4. Create and maintain one shared database pool for the selected backend.

For each REST request, the service shall:

1. Resolve the route or operation.
2. Resolve the configured SQL query from the loaded query definitions.
3. Map incoming request data to query inputs.
4. Acquire a short-lived database connection from the shared pool.
5. Execute the SQL query.
6. Map the result set to the configured response shape.
7. Return the response to the caller.

## Database Design Requirement

The database layer shall be optimized for pooled connections only. Single persistent connection support is intentionally excluded because it does not match the request-driven execution model of the service.

The required split is:

1. `DatabasePool`: long-lived application state shared by the process.
2. `DatabaseConnection`: short-lived connection acquired per request.
3. `ExecutionContext`: optional per-request execution wrapper for transactions, logging, metrics, and metadata.

This separation keeps connection resolution independent from query execution while allowing future execution logic to add transactions, result mapping, logging, and metrics without coupling them to connection lifecycle management.

## Connection Layer Requirement

The database connection module shall be limited to connection-related responsibilities only. It shall:

1. Build backend-specific pools from configuration.
2. Hold long-lived backend pool instances.
3. Acquire short-lived backend connections.
4. Expose a unified acquired connection type.

It shall not execute queries, resolve route logic, or contain result-mapping behavior.

## Runtime Expectations

The service shall run as a deployable application in a Docker-style environment where settings, SQL queries, and mappings are provided externally at runtime.

The expected runtime shape is:

- configuration loaded at process start
- shared DB pool created once and reused for the lifetime of the process
- request-driven execution flow for each REST call
- stateless query execution model built on pooled connections

## Repository Direction

The repository should evolve toward this architecture by keeping:

1. Application settings loading and validation under `src/context_container/app_settings/`.
2. Database backend modules for PostgreSQL and SQLite under `src/db/`.
3. Pooled connection abstractions and factory resolution under `src/db/`.
4. Docker-friendly test resources under `test-utils/` and `unit-test-resources/`.

The target design is a pooled-only service architecture aligned to the requirement above.