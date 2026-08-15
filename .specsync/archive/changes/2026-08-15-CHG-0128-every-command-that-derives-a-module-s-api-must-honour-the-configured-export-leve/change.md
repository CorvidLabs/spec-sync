---
id: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
state: archived
type: bug_fix
base_commit: 48caf5688f9e0b7b2096b33c8e6a5ae47897daa4
---

# Every command that derives a module's API must honour the configured export level and parse mode, so check, score, new, generate, scaffold and diff cannot disagree about what the API is

## Intent

Every command that derives a module's API must honour the configured export level and parse mode, so check, score, new, generate, scaffold and diff cannot disagree about what the API is

## Affected Canonical Specs

- `scoring`
- `generator`
- `exports`
- `cmd_new`
- `cmd_scaffold`
- `cmd_diff`

## Acceptance Criteria

- On a project configuring export_level = type, score grades the API dimension against the type-level surface that check validates, so the two commands agree about what the module's API is. new and generate produce specs whose Public API contains only symbols check will accept, so the tool cannot generate work its own validator rejects. diff reports drift only for symbols in the configured surface. The configured parse mode is threaded to the same sites, so an AST project gets AST parsing everywhere rather than in check alone. A project on the default member level and regex mode behaves byte-identically to before.

## No-spec Rationale

Not applicable
