# SQL Query Contract Feature

![Status: Partially implemented](https://img.shields.io/badge/status-partially%20implemented-yellow)

## Summary

User-defined SQL files should expose explicit inputs and outputs rather than
depend on backend-specific parameter syntax or implicit result column names.
This contract supports predictable query preparation and later result mapping.
It does not require protobuf support.

Query preparation and contract validation are implemented; executor
integration is deferred.

## Scope

In scope:

- canonical `$name` input placeholders in query files
- explicit, unique output aliases for the final result
- the naming grammar for inputs and output aliases
- the supported statement set for a single query file
- preparation that returns backend SQL and an ordered parameter vector

Out of scope for this document:

- binding and running a prepared query, covered by the [query executor feature](query-executor-feature-reference.md)
- classification, type resolution, and nullability, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- protobuf messages, covered by the [query-to-protobuf proposal](query-to-protobuf-feature-reference.md)

## Agreed requirements

### 1. Inputs use `$name`

Every input placeholder must have a `$` prefix, for example `$user_id` or
`$dataset_name`. This is sqlbinder's query-file convention for both PostgreSQL
and SQLite, not a claim that named placeholders are standard SQL.

The supplied parameter map uses the name without `$`. Values remain separate
from SQL text; preparation must not interpolate values into SQL.

Backend-native alternatives such as `$1`, `?`, `?1`, `:name`, and `@name` are not
accepted as input placeholders in query files. Validation must distinguish
placeholders from legitimate SQL syntax, including PostgreSQL `::` casts.
Tokens such as PostgreSQL's `?` operator must not be rejected merely because
they share spelling with another backend's placeholder syntax.
Likewise, PostgreSQL `@amount` is an absolute-value expression referring to a
column, not a bound input. To supply an input, use `$amount`.

Placeholder-like text in strings, quoted identifiers, dollar-quoted strings,
and comments is not an input. A `$` inside an SQL identifier is not automatically
a placeholder.

### 2. Outputs are explicit

Every column or expression exposed by the final `SELECT` result must use
`AS output_name` to declare its result name. Outputs are aliases, not `$`
placeholders. Output names must be unique within the final result.

Mandatory output aliases do not apply to projections inside CTEs and subqueries.
Those projections may still need names for SQL references, but only the final
result defines the external output contract.

Wildcard output projections are forbidden, including `SELECT *` and
`SELECT table_alias.*`. The output shape must be declared in the SQL file rather
than change implicitly when a table gains or loses columns. This restriction
also applies inside CTEs and subqueries.

`COUNT(*) AS item_count` is allowed: it produces one named value, not a wildcard
output projection. Multiplication is also allowed, for example
`price * quantity AS total_price`.

### 3. Names follow one grammar

Input names and final output aliases must match `[a-z][a-z0-9_]*`: start with a
lowercase ASCII letter, followed by lowercase letters, digits, or underscores.
Use lowercase snake_case, for example `$user_id` and `AS total_count`. Names are
case-sensitive and are not silently normalized.

Quoted output aliases, uppercase letters, leading underscores, and non-ASCII
characters in contract names are rejected initially. This restriction does not
apply to database object identifiers or dollar-quote tags.

### 4. One file contains one supported statement

Each SQL file contains exactly one SQL statement. An optional trailing semicolon
and surrounding comments are allowed; semicolons inside literals or comments
are not statement separators.

The initial supported statements are `SELECT`, `INSERT`, `UPDATE`, and `DELETE`,
including CTE-prefixed forms supported by the selected backend. Other statement
types are rejected explicitly rather than accepted without contract validation.

Writes without `RETURNING` have no named output fields; the later executor
integration will return an affected-row count for them. A final `RETURNING`
projection follows the same explicit-alias, unique-name, naming-grammar, and
no-wildcard rules as the final `SELECT` projection.

`UNION`, `INTERSECT`, and `EXCEPT` are initially unsupported, including inside
CTEs and subqueries. Validation must return an explicit unsupported-query error.
Supporting them later requires defined output-shape validation.

### 5. Preparation returns a bound query

The query handler loads a SQL file, validates its contract and supplied inputs,
and returns backend SQL plus an ordered parameter vector.

- PostgreSQL placeholders become `$1`, `$2`, and so on.
- SQLite placeholders become `?1`, `?2`, and so on.
- Distinct names are numbered in first-occurrence order.
- Repeated uses of a name reuse the same index and value.
- Missing and unused supplied parameters are rejected.
- Preparation errors are reported explicitly, not returned as successful queries.

Changing `QueryExecutor` to consume the bound query is separate, later work.
The query handler does not execute SQL.

## Example contract

```sql
SELECT
    u.id AS user_id,
    u.name AS user_name
FROM users AS u
WHERE u.id = $user_id
```

With `user_id = 7`, preparation produces the same SQL with `$1` for PostgreSQL
or `?1` for SQLite, and a parameter vector containing `DbValue::Int64(7)`.
Table aliases such as `AS u` are separate from output aliases.

## Current implementation

[`prepare_query`](../../src/db/query_handler/handler.rs) currently loads a file,
tokenizes and parses it with the selected backend's `sqlparser` dialect,
validates the SQL contract and supplied parameter names, and returns
[`BoundQuery`](../../src/db/query_handler/model.rs) with SQL and parameters.
The validator enforces statement count, supported statement types, contract
name grammar, explicit and unique final aliases, wildcard rejection, and
set-operation rejection. `SELECT INTO` is also rejected because it does not
provide the supported read-result contract.

The binder implements backend numbering and repeated-name reuse. Original SQL
text is retained, with only recognized input placeholders rewritten. Tests cover
non-ASCII dollar-quote tags, literals, comments, casts, operators, and byte offsets
with UTF-8 text and CRLF line endings.

The existing executor accepts raw SQL only and does not bind `BoundQuery`
parameters. SQL validation is limited to parser-supported syntax and project
rules; it does not establish that a query will execute successfully.

## Implementation approach

The implemented pipeline is **load -> tokenize/parse -> validate -> bind ->
return `BoundQuery`**.

`sqlparser` provides tokens with source locations and an abstract syntax tree
using the PostgreSQL or SQLite dialect. Both accept canonical `$name` inputs,
so no parser-only placeholder translation is needed. Name validation uses
character checks equivalent to the anchored pattern `^[a-z][a-z0-9_]*$`;
regular expressions are not used for structural SQL validation.

The preparation flow:

1. Load SQL, reporting file errors and rejecting empty content.
2. Tokenize and parse without changing literal or comment text; retain byte
   offsets for binding.
3. Validate the parsed contract and reject noncanonical input placeholders while
   preserving legitimate backend operators. Token positions also establish that
   final aliases use explicit `AS`, rather than implicit alias syntax.
4. Validate missing and unused supplied values.
5. Rewrite recognized inputs and return backend SQL and the ordered parameter vector.

Parsing alone does not verify database object existence, result types, or
nullability; those remain outside this feature.

## Acceptance coverage for implementation

Tests should cover backend rewriting, deterministic parameter order, repeated
inputs, missing and unused values, and rejection of noncanonical placeholders.
They should also cover:

- valid and invalid input names and output aliases, with no silent normalization;
- missing `AS`, quoted aliases, and duplicate names in final outputs;
- final-result alias enforcement without requiring aliases on internal projections;
- wildcard rejection in final outputs, CTEs, and subqueries;
- accepted `COUNT(*)` and multiplication expressions with explicit final aliases;
- one statement with optional trailing semicolon and comments, and multiple-statement rejection;
- writes with and without `RETURNING`, including its output validation;
- explicit rejection of unsupported statements and set operations at any nesting level.

SQL token tests must preserve strings, identifiers, comments, dollar quotes,
casts, and other backend syntax without interpreting their contents as inputs
or output projections. Output checks must understand projection structure;
a text search for `*` or `AS` is not sufficient.

## Related features

The [query manifest proposal](query-manifest-feature-reference.md) builds on
this contract: it records every validated query with its classification,
types, and nullability. The [query-to-protobuf proposal](query-to-protobuf-feature-reference.md)
generates typed messages from that manifest. This SQL contract defines input
and output naming independently; types, nullability, schema inspection, and
protobuf generation remain outside this feature's current scope.
