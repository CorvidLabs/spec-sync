---
id: complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1
state: implementing
type: bug_fix
base_commit: f72f0b6dd5c2fbf34617acda372d1e409c9f6b66
---

# Complete exact Cargo source handling and scoped batch recognition for SpecSync 6.1.1

## Intent

Complete exact Cargo source handling and scoped batch recognition for SpecSync 6.1.1

## Affected Canonical Specs

- `cmd_generate`
- `cmd_new`
- `cmd_scaffold`
- `generator`
- `watch`
- `github`

## Acceptance Criteria

- Scoped batch reruns classify existing leaf specs without overwrite; new/add-spec/scaffold select exact configured source files with filters and confinement; watch selects regular exact files nonrecursively and preserves recursive directories with truthful diagnostics; GitHub testing row states candidate6.1.1 and baseline6.1.0; focused regressions and full pinned verification pass before review and release

## No-spec Rationale

Not applicable
