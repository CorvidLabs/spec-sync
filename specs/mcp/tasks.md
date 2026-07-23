---
spec: mcp.spec.md
---

## Tasks

(none open for 5.0)

## Done

- [x] Make the server read-only by default and gate mutators behind `--allow-write`
- [x] Confine reads to canonical descendants and writes to the canonical server root
- [x] Enforce exact tool schemas and notification-safe no-dispatch behavior
- [x] Add adversarial outside-victim byte-preservation integration tests
- [x] Preflight manifest workspaces, metadata/cache/dependency paths, and bounded autodetection scans
- [x] Deterministic check, generate, coverage, score, list, init, and issue tools
- [x] Static resources and module resource template
- [x] Reject retired inference arguments without value disclosure
- [x] Reject invalid request envelopes before dispatch and validate resource arguments exactly
- [x] Bound project inputs and MCP responses, and surface transport/write failures
- [x] Disable Git metadata auto-detection in MCP issue verification
- [x] Detect generation collisions and incomplete writes without false success
- [x] Retain server-root capabilities and snapshot reads to close path-replacement races
- [x] Count configuration and actual copied bytes in one operation budget
- [x] Preserve explicitly configured roots that use normally ignored directory names
- [x] Mark snapshot Git freshness unavailable and withhold unverified score points
- [x] Roll back generated outputs when a later batch destination fails
- [x] Characterize stdin and stdout transport failures
- [x] Bind startup root acquisition to its pre-open filesystem identity
- [x] Preserve root-wide and manifest-derived inputs across snapshot ignore boundaries
- [x] Bound, stage, sync, atomically publish, and roll back generated output
- [x] Add Windows write-junction and oversized request-ID regressions
- [x] Capture the initial root identity before canonicalization and reject a mismatched reopen
- [x] Retain empty parents on failed batches rather than claim ownership across create/open races
- [x] Parse Cargo workspace members as TOML and charge manifest discovery to the shared budget
- [x] Fail closed and bound globally deduplicated GitHub issue verification
- [x] Copy immutable preflighted manifests and parse comment/escape-aware Gradle settings through one shared parser
- [x] Bind generated publication/rollback to retained parent capabilities and filesystem identities
- [x] Eliminate issue-read provider subprocesses and revalidate repository access after ambiguous not-found responses
- [x] Preserve public replacement entries across staging, publication, and rollback quarantine
- [x] Normalize drive, extended-drive, and UNC Windows roots case-insensitively
- [x] Add exact generated-byte and final response-preflight boundary regressions
- [x] Skip ignored and configured-exclusion symlink names before following their targets
- [x] Use native Windows ordinal Unicode case comparison for confined absolute roots
- [x] Pass independent defensive agent rereview with no high or medium findings
- [x] Bind publication and rollback identity to exact staged bytes to reject immediate inode reuse
- [x] Bound exact-byte identity hashing and fail closed on oversized replacement input
- [x] Preserve confined Cargo sibling dependencies that use manifest-relative parent components
- [x] Consume the final quarantine directory capability before Windows cleanup
- [x] Restrict Cargo snapshot paths to semantic target/workspace/dependency tables and ignore
  unrelated metadata `path` keys
- [x] Accept confined Windows-native Cargo backslashes while rejecting drive, UNC, rooted,
  traversal, symlink, and junction escapes
- [x] Fail MCP issue verification inconclusive on unreadable specs or malformed frontmatter
- [x] Repair Windows junction/read-root fixtures with native joins and a valid child project
- [x] Use checked spec traversal and checked top-level `implements`/`tracks` shapes in MCP issue
  verification while ignoring nested extension keys and block-scalar text
- [x] Return sanitized relative, content-free MCP diagnostics for spec discovery/read failures,
  including walker failures and non-UTF-8 filenames
- [x] Share maintained real-YAML issue parsing with CLI, rejecting duplicate/global malformed YAML
  and blank/null/wrong shapes while accepting comments/trailing commas and ignoring nested
  extension/block-scalar lookalikes
- [x] Preserve valid issue references in CRLF specs through the shared checked parser
- [x] Reject malformed, invalid-UTF-8, or wrong-typed selected config snapshots before MCP
  compatibility loading, including allow-empty tools and resources
- [x] Reject linked/reparse, non-regular, blocking, replaced, and non-object selected configs
  before MCP tools or resources can return an allow-empty success
- [ ] Pass fresh Windows reparse-point CI
