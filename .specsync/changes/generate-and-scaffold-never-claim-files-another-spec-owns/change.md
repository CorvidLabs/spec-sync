---
id: generate-and-scaffold-never-claim-files-another-spec-owns
state: implementing
type: bug_fix
base_commit: 49d2326a8f80232b8fee94a5073c4c1a66fd2057
---

# Generate and scaffold never claim files another spec owns

## Intent

generate and scaffold never claim files another spec owns

## Affected Canonical Specs

- `generator`
- `cmd_generate`
- `cmd_scaffold`

## Acceptance Criteria

- scaffold/new/generate filter candidate files against files already mapped in existing specs' frontmatter; a module whose files are all owned produces no spec; the MCP generate path behaves identically

## No-spec Rationale

Not applicable
