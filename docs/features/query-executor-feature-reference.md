# Query Executor Feature

![Status: Partially implemented](https://img.shields.io/badge/status-partially%20implemented-yellow)

## Summary

The query executor runs SQL against the configured PostgreSQL or SQLite pool
and returns either rows or an affected-row count.

Today it executes **raw SQL strings**. That remains supported for now. Raw
execution will later be replaced by **bound execution**: the executor will
take a [`BoundQuery`](../../src/db/query_handler/model.rs) prepared from a
registered query, bind its parameters, and map results to the query's declared
outputs.

## Scope

In scope:

- executing raw SQL (current) and bound queries (planned)
- binding parameter values
- mapping database rows to `DbValue`
- result shapes, errors, and execution limits
- the move from raw to bound execution

Out of scope for this document:

- SQL validation and binding rules, covered by the [SQL query contract](sql-query-contract-feature-reference.md)
- query registration and classification, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- protobuf messages, covered by the [query-to-protobuf proposal](query-to-protobuf-feature-reference.md)
- transports, authentication, and auditing, covered at a high level in the
  [project overview](../project_overview.md#architecture)

## Current implementation: raw SQL

### 1. API

[`QueryExecutor`](../../src/db/query/executor.rs) is created from a
`DbConnectionPool` and exposes:

- `execute_read(sql)` returns [`QueryRows`](../../src/db/query/model.rs):
  column names plus rows of `DbValue`
- `execute_write(sql)` returns the affected-row count as `u64`

The executor dispatches to a PostgreSQL or SQLite backend executor behind an
internal `BackendQueryExecutor` trait.

### 2. Behaviour

- SQL is executed as raw text through sqlx `raw_sql`. There are no bind
  parameters; the caller is responsible for the SQL being safe.
- A raw string may contain more than one statement.
- Reads fetch all rows into memory before returning.
- Errors are reported as `DbQueryError::Execution`, wrapping the sqlx error.

### 3. Known gaps

These gaps are acceptable for raw execution today, but bound execution must
close them:

| Gap | Effect |
|---|---|
| No bind parameters | Values must be embedded in SQL text |
| Multiple statements accepted | A raw string is not limited to one statement |
| Unknown column types become `DbValue::Null` | A decode failure looks like a real database null |
| Empty results return no column names | The output shape is lost when there are no rows |
| SQLite booleans come back as `DbValue::Int64` | Type depends on the backend, not on the contract |
| All rows are buffered | Memory grows with result size |
| No timeouts or result limits | A slow or large query is not bounded |
| `QueryResult` is exported but unused | The read/write result type is not used by the executor |

## Planned: bound execution

### 4. Flow

```text
registered query + input values
           |
     prepare_query  ->  BoundQuery (backend SQL + ordered parameters)
           |
     query executor
     bind -> execute -> map
           |
     rows or affected-row count
```

The [`prepare_query`](../../src/db/query_handler/handler.rs) step already
exists. Bound execution connects its output to the executor.

### 5. Rules

- The executor accepts a `BoundQuery` and binds each `DbValue` in order;
  values never become SQL text.
- One bound query runs exactly one statement.
- The result shape comes from the query's declared outputs, so an empty result
  still has its column names.
- A value that cannot be decoded to its declared type is an error, not a null.
- Values are mapped by declared type, so the same query returns the same types
  on every backend.
- Writes without `RETURNING` return an affected-row count; reads and writes
  with `RETURNING` return rows.

### 6. Limits

Every execution is bounded by:

- a timeout that cancels the query and releases its connection
- a maximum result size
- the pool's connection limit

Defaults and configuration are still open (EX-1).

### 7. Read-only execution

Reads should be enforced at the database as well as by classification, for
example through separate read-only credentials or read-only transactions. The
approach is still open (EX-3).

## Moving from raw to bound

1. **Now:** raw execution is the only path and stays supported.
2. **Next:** add bound execution alongside raw execution, close the known gaps
   for the bound path, and cover both backends with tests.
3. **Later:** bound execution becomes the default. Raw execution is
   deprecated, then restricted to internal or test use, or removed (EX-4).

Raw execution is never exposed to API callers. Only registered queries run
through the API.

## Future

- **Streaming** for large results: fetch and return rows incrementally, with
  backpressure and cancellation. Pagination comes first (EX-2).
- **Transactions** spanning more than one query, if writes need them.

## Open questions

These questions are deliberately left open until they are needed. When one is
answered, record the decision in the relevant section above and move it to
**Answered**.

| ID | Question | Needed by |
|---|---|---|
| EX-1 | What are the default timeout, result-size, and concurrency limits, and how are they configured? | Bound execution |
| EX-2 | How are large results paginated, and when is streaming introduced? | Large-result support |
| EX-3 | How is read-only execution enforced at the database: separate credentials, read-only transactions, or both? | Bound execution |
| EX-4 | Is raw execution removed, or kept for internal and test use only? | Bound execution becomes default |
| EX-5 | Does the executor return `QueryResult`, or keep separate read and write methods? | Bound execution |
| EX-6 | Are multi-query transactions supported, and how are they declared? | Write support |

### Answered

None yet.
