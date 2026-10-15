# Roadmap

Updated: 2026-10-14

This file records high-level product ideas and direction, not detailed scope or progress. Release documents under `docs/releases/` define committed scope, acceptance criteria, validation evidence, and release progress. The [feature tracker](features.md) records implementation state; the [project overview](../project_overview.md) describes the product direction.

## v1

Establish one robust, tested SQL-first path: SQL files, query manifest, bound execution, and a REST API with JSON payloads.

Detailed scope and progress: [v1 release acceptance](../releases/v1.md).

## After v1

- Replace the shared static API key with service-session authentication and operation permissions.
- Explore protobuf contracts, REST with protobuf, and gRPC.
- Add admin endpoints, observability, and audit events.
- Develop packaging, delivery, and usage guidance, including a container base image.

Except for service-session authentication being intended for the next release after v1, these ideas have no assigned release. Detailed scope and progress belong in their release documents when scheduled.
