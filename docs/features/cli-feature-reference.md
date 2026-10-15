# Command-Line Interface Feature

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

The planned sqlbinder CLI exposes project initialization, manifest generation,
and server startup as explicit commands. This separates project setup and
build-time database introspection from the long-running runtime service.

The current executable only loads settings, creates a pool, and exits; none of
the commands in this proposal are implemented.

## Scope

In scope:

- one CLI entry point and dispatch for supported subcommands
- project-path selection and project settings loading
- `init`, `generate-manifest`, and `server` command contracts
- actionable errors and nonzero failure status for failed operations
- documented credential handling without password command-line arguments

Out of scope for this document:

- project scaffold contents, covered by the [project initialization feature](project-initialization-feature-reference.md)
- database schema inspection, covered by the [schema introspection feature](schema-introspection-feature-reference.md)
- manifest structure and compatibility, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- HTTP endpoints and request/response contracts, covered at a high level by the [project overview](../project_overview.md#architecture)
- container packaging and deployment automation, covered at a high level by the [project overview](../project_overview.md#architecture)

## 1. Planned command workflow

The intended workflow is:

```text
sqlbinder init <project-directory>
sqlbinder generate-manifest --project <project-directory>
sqlbinder server --project <project-directory>
```

These forms are illustrative; the exact argument syntax is not finalized.

- `init` creates the project template described in the [project initialization feature](project-initialization-feature-reference.md).
- `generate-manifest` loads project settings, connects to the configured database, inspects its schema, validates project queries, and writes the manifest.
- `server` loads project settings and the previously generated manifest, creates the runtime database pool, and starts the API. It does not generate or rewrite the manifest at startup.

## 2. Project context and paths

Commands that operate on a project must resolve settings, environment overlays,
queries, the manifest, and configured credential-file paths consistently from
the selected project root. Behavior must not silently change based on the
process working directory.

The final option names, default project path, manifest path override behavior,
and path traversal/symlink policy remain to be specified. The CLI must report
the selected project root and relevant input/output paths without exposing
secret contents.

## 3. Credentials and command failures

The initial database password source is a file configured in project settings,
with the scaffold using ignored `db.local.pwd`. The CLI must not accept a
database password as a normal argument or echo it in logs or diagnostics.
Container deployments may mount a secret at the configured file path.

Commands fail explicitly when required settings, files, connections, schema
metadata, SQL contracts, or manifest inputs are invalid. They return a
nonzero exit status on failure and actionable, redacted diagnostics. A failed
manifest generation must not leave a partial output or destroy the prior
manifest.

## 4. Open questions

| ID | Question | Needed by |
|---|---|---|
| CLI-1 | What exact flags, positional arguments, defaults, and help/version behavior are supported? | CLI implementation |
| CLI-2 | How are settings and environment overlays selected, and are command-specific configuration overrides supported? | CLI implementation |
| CLI-3 | What are the output, logging, and exit-status conventions for each command? | CLI implementation |
| CLI-4 | How does `server` report readiness, handle shutdown signals, and propagate startup/runtime failures? | Server integration |
| CLI-5 | Does `generate-manifest` support explicit database/schema overrides, or must all target selection live in settings? | Introspection implementation |

### Answered

- The planned user journey consists of `init`, `generate-manifest`, and `server`; the command spelling and arguments above remain illustrative.
- The initial credential source is a configured password file, not a password command-line argument.
- The initial file-only workflow reuses the existing `database.password_file` setting; provider selection can be revisited when another credential provider is implemented.
- Server startup consumes an existing manifest and does not regenerate it.
