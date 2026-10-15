# Query Manifest Design Proposal

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

The query manifest is the core of query management in sqlbinder. It is a
single generated, version-controlled file that describes every query in a SQL
directory: its identity, its operation type, and its typed inputs and outputs.

The planned project workflow has the contract tool connect to the configured
database and inspect its schema while generating the manifest from the SQL
files. A separately supplied schema-snapshot file is not required for this
workflow. Protobuf contracts are then generated from the manifest, not from the
SQL files directly. Everything downstream relies on the manifest as the single
description of what each query is.

Nothing in this document is implemented.

## Scope

In scope:

- discovering SQL files in a query directory
- classifying each query
- resolving input and output types from inspected database schema
- assigning stable field numbers
- the manifest's contents, determinism, and change tracking

Out of scope for this document:

- SQL validation rules, covered by the [SQL query contract](sql-query-contract-feature-reference.md)
- connecting to and inspecting database catalogs, covered by the [schema introspection feature](schema-introspection-feature-reference.md)
- generating protobuf messages from the manifest, covered by the [query-to-protobuf proposal](query-to-protobuf-feature-reference.md)
- runtime use, transports, authentication, and packaging, covered at a high
  level in the [project overview](../project_overview.md#how-you-would-use-it)

## Flow

```text
SQL directory + database schema + previous manifest
                       |
                 contract tool
     connect -> discover -> validate -> classify -> resolve types
                       |
                 query manifest
                       |
              protobuf generation
```

The manifest is generated, never edited by hand.

## Creating the manifest

### 1. Steps

1. **Discover** every `.sql` file under the configured query directory
   (`queries.directory`), including subdirectories.
2. **Validate** each file with the existing SQL contract rules: one supported
   statement, `$name` inputs, explicit unique output aliases, no wildcards.
3. **Classify** each query as `read`, `insert`, `update`, `delete`, or
   `upsert`.
4. **Inspect the database schema** and resolve types and nullability for every
   input and output. Schema selection and the metadata permissions required
   depend on the selected backend and remain to be specified.
5. **Assign field numbers**, reusing the numbers recorded in the previous
   manifest.
6. **Write** the manifest deterministically.
7. **Compare** it with the previous manifest and report breaking changes.

Any failure stops generation; a partial manifest is never written.

### 2. Type resolution

Types and nullability come from the inspected database schema, not from sample
query results. Types are resolved for:

- **inputs:** the column or expression each `$name` parameter is compared
  with or assigned to
- **outputs:** each final `SELECT` or `RETURNING` projection, including
  expressions

The manifest records types in a small, backend-neutral vocabulary. An
illustrative starting point:

| Database type | Manifest type |
|---|---|
| `INTEGER` / `BIGINT` | `int64` |
| `TEXT` / `VARCHAR` | `string` |
| `BOOLEAN` | `bool` |
| `REAL` / `DOUBLE` | `float64` |
| `BLOB` / `BYTEA` | `bytes` |

This is not a supported mapping table yet. Decimal, timestamp, UUID, JSON, and
integer-range rules must be specified before implementation (QM-2). A type that
cannot be resolved or is not supported stops generation with an explicit
error; it is never guessed.

### 3. Stable field numbers

Every input and output has a field number. Numbers are kept from the previous
manifest for fields that still exist, new fields get the next unused number,
and removed fields are kept as reserved entries so their numbers are never
reused. This makes the manifest the field-number history that protobuf
generation relies on.

## Manifest contents

### 4. Manifest-level fields

| Field | Purpose |
|---|---|
| `format_version` | Version of the manifest format |
| `backend` | Target database backend (`postgres` or `sqlite`); prepared SQL differs per backend |
| `queries` | One entry per SQL file, sorted by `id` |

### 5. Per-query fields

| Field | Purpose |
|---|---|
| `id` | Stable identity derived from the relative path without extension, for example `users/get_user` |
| `source` | Relative path of the SQL file |
| `source_hash` | Hash of the SQL file content, used to detect drift |
| `operation` | `read`, `insert`, `update`, `delete`, or `upsert` |
| `sql` | Prepared backend SQL with positional placeholders |
| `inputs` | Ordered inputs: name, bind position, type, nullability, field number |
| `outputs` | Outputs: name, type, nullability, field number; empty when there is no result row |
| `result` | `rows` when the query returns rows; `affected_rows` for writes without `RETURNING` |
| `reserved` | Field numbers and names of removed inputs and outputs |

Bind positions follow the existing binder: distinct names are numbered in
first-occurrence order, and repeated names reuse one position.

### 6. Example

Illustrative only; the exact schema is not fixed.

```json
{
  "format_version": 1,
  "backend": "postgres",
  "queries": [
    {
      "id": "users/get_user",
      "source": "users/get_user.sql",
      "source_hash": "sha256:...",
      "operation": "read",
      "sql": "SELECT u.id AS user_id, u.name AS user_name FROM users AS u WHERE u.id = $1",
      "inputs": [
        { "name": "user_id", "position": 1, "type": "int64", "nullable": false, "field_number": 1 }
      ],
      "outputs": [
        { "name": "user_id", "type": "int64", "nullable": false, "field_number": 1 },
        { "name": "user_name", "type": "string", "nullable": false, "field_number": 2 }
      ],
      "result": "rows",
      "reserved": { "inputs": [], "outputs": [] }
    }
  ]
}
```

### 7. What the manifest never contains

- parameter values
- credentials, keys, or connection settings
- generation timestamps or machine-specific paths

## Query classification

### 8. Operation types

| Operation | Statement |
|---|---|
| `read` | `SELECT`, including CTE-prefixed forms, with no data-modifying parts |
| `insert` | `INSERT`, including `ON CONFLICT DO NOTHING` |
| `update` | `UPDATE` |
| `delete` | `DELETE` |
| `upsert` | `INSERT ... ON CONFLICT ... DO UPDATE` |

`RETURNING` does not change the classification: a `DELETE ... RETURNING` is
still a `delete`.

### 9. Classification rules

- Classification uses the parsed statement, not its first keyword.
- The whole statement is inspected, including CTEs and nested queries. A
  data-modifying CTE makes the query a write, even when the final statement is
  a `SELECT`.
- A query that combines more than one write operation is rejected initially,
  rather than given a single misleading classification.
- Classification describes statement structure. It cannot detect side effects
  inside database functions; database privileges must cover those.

## Determinism and change tracking

### 10. Deterministic output

The same SQL files, observed database schema, and previous manifest always
produce the same manifest: queries sorted by `id`, fields in a fixed order,
and stable formatting. Git diffs on the manifest then show exactly when a
query's operation, inputs, outputs, or types change.

### 11. Breaking changes are reported

Comparing the new manifest with the previous one, the tool reports breaking
changes, such as a removed output, a changed type or nullability, or a changed
operation. Generation fails on a breaking change unless it is explicitly
allowed (QM-5).

### 12. Generation fails when

- two files produce the same `id`
- a file fails SQL contract validation
- a query cannot be classified
- an input or output type cannot be resolved
- a breaking change is not explicitly allowed

## Rule set

1. The manifest is the single generated description of all queries; protobuf contracts are generated from it.
2. One manifest per SQL directory and backend.
3. Every SQL file has one entry with a stable, path-derived `id`.
4. Every query is classified from its parsed structure.
5. Types and nullability come from the inspected database schema and are never
   guessed.
6. Field numbers are stable; removed fields are reserved.
7. Output is deterministic and contains no values, secrets, or timestamps.
8. Breaking changes are reported and fail generation unless allowed.

## Open questions

These questions are deliberately left open until they are needed. When one is
answered, record the decision in the relevant section above and move it to
**Answered**.

| ID | Question | Needed by |
|---|---|---|
| QM-1 | How does manifest generation consume inspected schema metadata, and should it also accept an optional exported snapshot? | Manifest implementation |
| QM-2 | Which type rules apply to decimal, timestamp, UUID, JSON, and integer ranges? | Manifest implementation |
| QM-3 | Which hash algorithm is used, and should hashes cover only SQL files or also the inspected schema? | Manifest implementation |
| QM-4 | Is the prepared `sql` stored in the manifest or re-derived from the source file? | Manifest implementation |
| QM-5 | How is a breaking change explicitly allowed during generation? | Manifest implementation |
| QM-6 | How does one SQL directory support more than one backend? | Multi-backend support |

### Answered

None yet.
