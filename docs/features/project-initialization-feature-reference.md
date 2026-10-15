# Project Initialization Feature

![Status: Planned](https://img.shields.io/badge/status-planned-blue)

## Summary

`sqlbinder init` is planned to scaffold the minimum files needed to start a
sqlbinder project. It creates a settings template, a query directory, local
password-file configuration, Git ignore rules, and project-specific
`AGENTS.md` guidance for coding agents.

Initialization is not implemented yet. The exact CLI syntax and full template
are still to be finalized.

## Scope

In scope:

- creating a project directory from a user-supplied path
- creating baseline settings and a local settings overlay
- creating an ignored local database password file
- creating a SQL query directory and `.gitignore`
- creating a version-controlled `AGENTS.md` with accurate workflow and safety
  guidance

Out of scope for this document:

- CLI argument parsing and command dispatch, covered by the [CLI feature](cli-feature-reference.md)
- inspecting database schema, covered by the [schema introspection feature](schema-introspection-feature-reference.md)
- generating the query manifest, covered by the [query manifest proposal](query-manifest-feature-reference.md)
- starting the API server, covered by the [product overview](../project_overview.md#architecture)
- database creation, migrations, or credential generation

## 1. Project template

The proposed minimal layout is:

```text
<project>/
├── .gitignore
├── AGENTS.md
├── settings.toml
├── settings.local.toml
├── db.local.pwd
└── queries/
```

`manifest.json` is generated later by `generate-manifest`; `init` must not
create a success-shaped placeholder manifest. An example SQL file may be useful
but is optional until its contents and inclusion are agreed.

## 2. Settings and local password file

`settings.toml` contains non-secret project defaults and selects the local
environment. `settings.local.toml` is the ignored environment overlay. Its
database `password_file` setting points to `db.local.pwd`. This uses the
existing file-based password setting; local project path resolution still
needs to be specified by the [CLI and project configuration contract](cli-feature-reference.md).

`init` creates the root-level `db.local.pwd` empty. Users supply their own password or mount a
container secret at the configured path. The scaffold must not generate a
placeholder password or print secret contents. The local overlay and password
file are excluded by the generated `.gitignore`; the settings template,
queries, manifest, and `AGENTS.md` remain eligible for version control.

The initial credential source is a file. The settings design should avoid
coupling the project format to a single provider so other mechanisms can be
considered later. Key vault and IAM providers are future possibilities, not
part of this initial scope.

## 3. Git and existing files

The generated ignore rules must cover at least `settings.local.toml` and
`db.local.pwd`. They must not ignore `manifest.json` or the SQL source files.
Initialization must not overwrite user files by default. Existing directories,
conflicting files, and whether an explicit force/merge mode is needed remain
open questions.

The generated `AGENTS.md` is intended to be checked into the user's project.
It should explain the supported CLI workflow, where queries and generated
artifacts belong, how to validate query changes, and how to use the local
password file without exposing it. It must not invent command behavior; the
[CLI feature](cli-feature-reference.md) is the source of the command contract.

## 4. Open questions

| ID | Question | Needed by |
|---|---|---|
| PI-1 | What exact `settings.toml` and `settings.local.toml` values should the template contain for each backend? | Template implementation |
| PI-2 | Should `init` include an example SQL query, and if so, what schema does it assume? | Template implementation |
| PI-3 | How should initialization behave for an existing or non-empty directory, and should overwrite/merge be supported? | CLI implementation |
| PI-4 | What exact content belongs in the generated `AGENTS.md`, and how will it stay aligned with CLI changes? | Template implementation |
| PI-5 | How are relative settings, overlay, and password-file paths resolved from the project root? | Project configuration |
### Answered

- The initial scaffold uses a password file configured in the local settings overlay; the password file is named `db.local.pwd` and ignored by Git.
- The initial file-only workflow reuses the existing `database.password_file` setting; no provider selector is required until another credential provider is implemented.
- Key vault and IAM integrations are not part of the initial file-based implementation scope.
