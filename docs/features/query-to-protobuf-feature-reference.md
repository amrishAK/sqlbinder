# Query-to-Protobuf Design Proposal

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

This document proposes generating protobuf message definitions from the
[query manifest](query-manifest-feature-reference.md). Each manifest entry
produces an input and response message, plus an output message only for
row-producing operations. Nothing in this document is implemented.

The proposed manifest would hold each query's resolved types, nullability, and
stable field numbers. The proposed protobuf generator would read only that
metadata; it would not re-read SQL files or inspect the database schema.

## Scope

In scope:

- deriving input, output, and response messages from a manifest entry
- mapping manifest types and nullability to protobuf field types
- naming messages
- validating generated messages against the manifest

Out of scope for this document:

- discovering SQL, classification, type resolution, and field-number
  assignment, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- SQL validation rules, covered by the [SQL query contract](sql-query-contract-feature-reference.md)
- Rust code generation, transports, authentication, and runtime use, covered
  at a high level in the [project overview](../project_overview.md#two-usage-modes)

## Message model

### 1. One manifest entry = one set of messages

Each query entry in the proposed manifest produces one input and one response
message. Row-producing operations also produce one output message; writes
without `RETURNING` do not. All generated messages go into one protobuf package.

### 2. Message sources

| Message | Source in the manifest entry |
|---|---|
| Input | `inputs` |
| Output | `outputs`, describing one result row |
| Response | `result`: a repeated list of output rows for `rows`, or an affected-row count for `affected_rows` |

Example: this SQL file

```sql
SELECT
    u.id AS user_id,
    u.name AS user_name
FROM users AS u
WHERE u.id = $user_id
```

has a manifest entry with one `int64` input and two outputs, which produces:

```proto
message GetUserInput {
  int64 user_id = 1;
}

message GetUserOutput {
  int64 user_id = 1;
  string user_name = 2;
}

message GetUserResponse {
  repeated GetUserOutput items = 1;
}
```

A write without `RETURNING` has no outputs, so it produces no output message;
its response carries an affected-row count:

```proto
message CreateUserInput {
  string name = 1;
  string email = 2;
}

message CreateUserResponse {
  int64 affected_rows = 1;
}
```

### 3. Field names and numbers

- Field names are the manifest's input and output names. The SQL contract
  already guarantees they are valid lowercase snake_case and unique.
- Field numbers are taken from the manifest as they are. The generator never
  assigns or renumbers them.
- The manifest's `reserved` entries become protobuf `reserved` statements, so
  removed numbers and names are never reused.

### 4. Response messages are typed per query

Each response message is specific to its query. The existing generic JSON
`count`/`items` envelope is not used as a source for message fields.

## Type mapping

### 5. Manifest type to protobuf type

| Manifest type | Protobuf type |
|---|---|
| `int64` | `int64` |
| `string` | `string` |
| `bool` | `bool` |
| `float64` | `double` |
| `bytes` | `bytes` |

This table grows with the manifest's type vocabulary. A manifest type with no
protobuf mapping stops generation with an explicit error.

### 6. Nullability

Nullable fields must be distinguishable from default values such as `0` or an
empty string. Whether to use proto3 `optional` or wrapper types is still open
(QP-1), and the choice must be applied consistently.

## Naming

### 7. Deterministic message names

Message names derive from the last segment of the query `id`:

- `users/get_user` -> `GetUserInput`, `GetUserOutput`, `GetUserResponse`
- `users/create_user` -> `CreateUserInput`, `CreateUserOutput` (with `RETURNING`), `CreateUserResponse`

Two queries in different folders with the same file name would produce the
same message names; how to resolve that is still open (QP-2).

## Validation

### 8. Generated messages must match the manifest

Generation checks that:

- every manifest input and output has a field with the same name and number
- every manifest type has a protobuf mapping
- nullability is represented as chosen
- no two queries produce the same message name

### 9. `buf` validates protobuf, not the query

`buf` can check generated definitions for syntax, lint rules, and breaking
changes. It does not check that messages match the queries; that is the
manifest's and the generator's job.

## Rule set

1. Protobuf messages are generated from the manifest only.
2. Each manifest entry produces one input and one response message, plus an output message only for row-producing operations; writes without `RETURNING` have no output message.
3. Field names and numbers come from the manifest; reserved entries become protobuf `reserved`.
4. Manifest types map to protobuf types through a fixed table; unmapped types fail generation.
5. Nullable fields are distinguishable from default values.
6. Message names are deterministic and must not collide.
7. `buf` validates protobuf; the manifest and generator ensure the messages match the queries.

## Open questions

These questions are deliberately left open until they are needed. When one is
answered, record the decision in the relevant section above and move it to
**Answered**.

| ID | Question | Needed by |
|---|---|---|
| QP-1 | Proto3 `optional` or wrapper types for nullable fields? | Protobuf implementation |
| QP-2 | How are messages named when query file names collide across folders, for example by using the folder as a package or name prefix? | Protobuf implementation |
| QP-3 | Do all queries share one `.proto` file, or does each get its own? | Protobuf implementation |

### Answered

None yet.
