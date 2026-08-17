# Query-to-Protobuf Feature Reference

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

This feature adds protobuf compatibility to the existing JSON-based runtime without requiring a service layer yet. The design treats SQL query definitions as the source contract for generated message schemas, while the runtime uses typed `QueryRows` as the internal contract boundary before serializing to either JSON or protobuf output.

The project already has a JSON-style envelope as a generic transport format. That remains useful for compatibility and debugging, but it is not the canonical source for strongly typed protobuf contracts. Protobuf responses should be generated from typed message definitions, not reconstructed from a lossy JSON envelope.

This is a message-first design, not an RPC-service-first design. There are no service definitions in scope yet; only generated message contracts and response serialization behavior.

## Core design

### 1. Canonical flow is DB -> Rust -> Message -> Output format

The runtime should define a fixed conversion pipeline:

- DB schema defines column types and nullability
- query result is represented as typed `QueryRows` in Rust
- Rust values are validated against a known DB-to-Rust mapping
- generated message types are populated from the validated values
- a formatter converts the message into either JSON or protobuf bytes

This pipeline is the contract boundary. It must be deterministic and versioned.

The project should maintain a predefined type matrix such as:

- `INTEGER` / `BIGINT` -> `i64` -> `int64`
- `TEXT` / `VARCHAR` -> `String` -> `string`
- `BOOLEAN` -> `bool` -> `bool`
- `REAL` / `DOUBLE` -> `f64` -> `double`
- `BLOB` -> `Vec<u8>` -> `bytes`
- nullable values must be represented as optional or nullable proto fields

This is the default mapping source of truth.

### 2. JSON remains a compatibility transport, not the proto contract

The current JSON envelope is a neutral, schema-agnostic payload:

- `count`
- `items[]`
- per-row key/value map of column name to scalar-like value

This remains useful for generic exchange, debug output, and compatibility with older clients.

However, it is intentionally not the canonical protobuf contract. The protobuf path should instead produce typed messages, optionally wrapped in a typed response envelope that matches the generated proto schema.

That means:

- JSON envelope = generic transport wrapper
- proto response = typed generated message or typed response envelope
- message fields must not be derived from a lossy generic envelope

### 3. Strongly typed mapping should use `QueryRows`

For message generation and row mapping, the preferred source is the actual query result (`QueryRows`):

- columns are known
- row values are typed at the DB boundary
- validation can happen before message construction
- mapping logic is explicit and predictable
- conversion rules are fixed instead of ad hoc

This is the better base for:

- repeated message generation
- typed protobuf serialization
- validation of output names and types against the target proto schema

## Query contract rules

### 4. One SQL file = one query contract

The project should enforce this rule:

- one `.sql` file contains one query contract
- one query generates one main contract
- one file should not hide multiple query shapes

This keeps generated messages and runtime behavior easy to reason about.

For multi-query or procedure-driven flows, the system may redirect to stored procedures or dedicated query entry points, but those should still be explicit contracts rather than implicit mixed results.

### 5. Query column aliases should match message field names

The simplest and most reliable rule is:

- SQL output aliases must match the protobuf field names
- for joins or computed values, alias names must be explicit

Example:

```sql
select
  u.id as id,
  u.full_name as name,
  u.is_active as active
from users u
```

This maps naturally to:

```proto
message User {
  int64 id = 1;
  string name = 2;
  bool active = 3;
}
```

This should be the default behavior.

### 6. Manual mapping is a fallback, not the default

Manual mapping only applies when the query cannot be expressed in a message-shaped form, such as:

- legacy query compatibility
- columns that do not match field names
- derived values
- joins with repeated or conflicting names
- stored procedure output shapes that differ from the named contract

The external mapping file can hold exceptions, but the primary flow should still prefer alias-based alignment and schema-driven typing.

## Validation model

### 7. Validation comes from DB schema, column definitions, and alias definitions

Validation must not be based only on message names or a generic transport wrapper. It should be driven by the actual source contract:

- database schema defines column existence and type
- column definitions define nullability and constraints
- query aliases define the output names used by the generated message
- generated proto fields must match the validated output contract

This should happen before message construction, not during row iteration.

The validation checklist should include:

- every required field in the message exists in the query output
- no duplicate output names exist
- aliases match the expected names or mapping rules
- DB types are compatible with the defined Rust and proto conversion rules
- nullability expectations are acceptable
- generated message fields align with the query contract

### 8. `buf` validates generated proto, not the SQL contract

`buf` is useful for validating generated protobuf definitions:

- syntax validity
- linting
- breaking change detection
- generated code consistency

`buf` does not validate whether the SQL result actually matches the intended proto structure. That must be the project’s own runtime validation step based on the DB schema and query definition.

## Message generation flow

### 9. Build-time generator creates proto definitions from SQL contracts

A generator tool reads each SQL definition and produces the corresponding proto contract.

Example:

- `get_user.sql` -> `GetUserResponse`
- `create_user.sql` -> `CreateUserRequest`
- `update_order.sql` -> `UpdateOrderRequest`
- `delete_order.sql` -> `DeleteOrderRequest`

The tool should:

1. parse the SQL contract
2. inspect the columns, parameters, and aliases
3. validate names and types against schema metadata
4. generate the `.proto` file and mapping metadata
5. invoke `buf` or `protoc` to generate the Rust model types

This is a generation step, not runtime execution logic.

### 10. Read queries generate response messages

For read queries, the output columns define the generated message.

Example:

- `get_user.sql`
- output columns: `id`, `name`, `email`
- generated type: `GetUserResponse`

The runtime pattern is:

1. execute query
2. read `QueryRows`
3. validate output columns against DB schema and alias contract
4. map each row into one generated message
5. serialize that message to JSON or protobuf bytes

### 11. Write queries generate request messages

For write queries, the request parameters define the generated message.

Example:

```sql
insert into users (name, email)
values (:name, :email)
```

Generated request message:

```proto
message CreateUserRequest {
  string name = 1;
  string email = 2;
}
```

If the write query includes a `RETURNING` clause, a separate response message should also be generated from the returned columns.

Thus the pattern becomes:

- write query -> request message from bind parameters
- write query with `RETURNING` -> response message from returned columns

### 12. CRUD is primarily a message concern

The system is primarily designed around typed CRUD operations in protobuf form:

- create request
- read response
- update request
- delete request/response as needed

This keeps the generated proto contract aligned with actual database operations and message semantics.

The system does not define an RPC/service layer yet; it defines the message contract and response serialization behavior first.

### 13. Response envelope may exist, but it is typed per proto contract

A response envelope is acceptable when there is a real API need for it, but it should be defined at the proto layer, not reused from a generic JSON envelope model.

Examples:

```proto
message GetUsersResponse {
  repeated User items = 1;
  int64 count = 2;
}
```

or

```proto
message CreateUserResponse {
  User item = 1;
}
```

The envelope is therefore message-specific and typed, not a universal generic transport class.

### 14. Naming convention should be deterministic

A file-based naming rule is recommended:

- `get_user.sql` -> `GetUserResponse`
- `create_user.sql` -> `CreateUserRequest`
- `update_order.sql` -> `UpdateOrderRequest`
- `delete_order.sql` -> `DeleteOrderRequest`

The exact naming convention should be fixed and enforced so users do not need to hand-maintain message names.

## Response formatting layer

### 15. Formatter receives a format type and returns the serialized payload

The formatter layer decides how the validated message is emitted:

- JSON output -> string
- protobuf output -> bytes

This allows the runtime to keep one internal message-building path while supporting multiple transport encodings.

Example contract:

```rust
enum ResponseFormat {
    Json,
    Proto,
}

fn format_response<T>(message: T, format: ResponseFormat) -> Result<ResponsePayload, Error>
where
    T: SerializeOrProto,
```

The payload should be one of:

- JSON string for generic transport responses
- protobuf bytes for typed message responses

This layer is responsible for serialization only. It is not the source of the contract.

### 16. Service definitions are intentionally out of scope

At this stage, the project is not yet defining a gRPC or RPC service layer. The contract is message-driven:

- SQL defines output structure
- generated proto defines the message schema
- runtime builds message instances
- formatter serializes the message to JSON or protobuf bytes

This keeps the scope small and aligned with the current repository, which already centers around `QueryRows` and formatter-level output shaping.

## External mapping file

The external file is used to define the relationship between:

- query identity
- generated message name
- bind fields or output fields
- optional overrides or type conversions
- required or optional status
- response wrapper behavior

The file should define metadata such as:

- query name
- source SQL file
- target message name
- field-to-column mapping
- field type
- required or optional status
- conversion rules when needed

This file is the contract registry, not the source of truth for the actual query logic. The source of truth remains the SQL definition, database schema, and validation metadata.

## Migration and schema management

### 17. Migrations are maintained separately

Database migration tracking is not part of the proto generation flow.

The design should keep these concerns separate:

- migrations define schema evolution
- SQL contract defines query shape
- proto generation defines typed request/response messages
- runtime validation checks compatibility between the above

This separation reduces drift and keeps schema changes explicit.

## Recommended rule set

This is the recommended operating model:

1. one SQL file = one query contract
2. one query = one generated message source
3. DB schema and column metadata are the validation source of truth
4. query aliases should match proto field names by default
5. DB -> Rust -> message -> output format is the conversion pipeline
6. read queries generate response message definitions
7. write queries generate request definitions
8. `RETURNING` queries also generate response definitions
9. message contracts are the primary interface abstraction
10. JSON envelope remains a generic transport option
11. proto envelopes are typed and message-specific
12. migrations are managed separately
13. validation happens before row mapping
14. `buf` validates generated proto, but not SQL-to-proto compatibility
15. the formatter returns serialized output based on a selected format kind
16. service definitions are intentionally not included in this phase

## Practical recommendation

If the goal is lower user load, the best workflow is:

- user writes SQL
- user keeps alias names stable and message-shaped
- database schema defines the actual data contract
- generator derives the proto message and mapping metadata
- runtime validates the query output against database schema and alias metadata
- generator produces Rust models via `buf` or `protoc`
- formatter serializes the resulting message to JSON or protobuf bytes

This reduces manual work without losing contract correctness.

## Design conclusion

The strongest approach is to map directly from typed `QueryRows` into generated message models using a predefined DB-to-Rust-to-Proto conversion table. The JSON envelope remains useful as a generic transport format, while protobuf responses use typed, message-specific definitions generated from the SQL contract.

The runtime does not need a service layer to begin with. A message-first design is sufficient for this repository: build the query contract, generate the message schema, validate the result, map to the generated message, and serialize using either JSON or protobuf depending on the requested format.

For write operations, generate the request message from bind parameters; for return-bearing writes, generate the response message from the `RETURNING` columns. This aligns the query contract with actual database behavior and keeps the workflow simple, predictable, and schema-driven.
