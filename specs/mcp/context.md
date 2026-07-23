---
spec: mcp.spec.md
---

## Key Decisions

- MCP is a deterministic stdio JSON-RPC adapter for coding agents.
- Generate creates local templates only and rejects retired inference arguments.
- Tool errors use `isError`; protocol errors remain JSON-RPC errors.
- Agent credentials and model execution stay outside SpecSync.
- The retained server-root handle is the filesystem authority boundary. Read roots are lexically validated descendants opened only through that handle.
- Mutation is an explicit server capability; mutators are hidden and denied unless `--allow-write` is present, and never accept per-call roots.
- Parsed notifications are discarded before dispatch so they cannot trigger filesystem work.
- Confinement is enforced again at downstream path sources: config/metadata/cache files, configured
  path fields, Cargo/package/Gradle/Python autodetection paths, dependency references, module
  names/files, spec mappings, nested symlinks, generated destinations, and init destinations.
- Recursive checks canonicalize only symlinks, honor excluded directories, and stop at deterministic
  entry/manifests bounds; ignored or configured-exclusion symlink names are skipped before target
  metadata is followed unless an explicit configured input names them or a descendant; a broad
  ancestor such as `.` does not override `excludeDirs`. No-config source
  autodetection is preflighted to the same four-level scope.
- Absolute outside roots are rejected before canonicalization, project inputs and protocol outputs
  have byte budgets, and only valid JSON-RPC 2.0 envelopes reach dispatch.
- MCP issue verification requires explicit `github.repo`; it never follows project-controlled Git
  metadata. Read operations run from a bounded capability snapshot, while writes use the retained
  server-root capability so path replacement cannot redirect an operation.
- Configuration bytes and actual copied bytes share one cumulative operation budget. Explicitly
  configured source roots remain visible even when their basenames are normally ignored.
- Git contents are excluded from snapshots. MCP score output marks Git freshness unavailable and
  conservatively withholds its five points instead of returning a false-high score.
- Generation verifies every required destination, reports incomplete writes, returns relative
  project paths, bounds count/content/result size before mutation, stages and syncs content beside
  each destination, and atomically publishes without overwrite. Rollback preserves replacements
  at public transaction paths. File identities include bounded exact-byte digests so immediate inode reuse
  or same-entry rewrites cannot authorize a replacement. Empty parents created by a failed batch may remain because portable
  filesystems provide no atomic create-and-open directory primitive. A same-user process already
  authorized to mutate the server root must not race private staging or quarantine names.
- Quarantine cleanup consumes the final identity-checked directory capability before removal;
  cloning that capability leaves a sharing-blocking original handle on Windows.
- Root capability acquisition opens and identity-binds the requested root before canonicalization,
  reopens the canonical path through its parent capability, and compares filesystem identities,
  closing the full startup replacement interval.
- Root-wide configuration and manifest-derived workspace paths override fixed snapshot ignores so
  required inputs cannot disappear into false-green analysis. Manifest discovery parses Cargo
  workspace membership as TOML and shared Gradle settings with comment/escape-aware syntax,
  charges deduplicated manifest bytes to the operation budget, and copies those exact preflight
  buffers.
- Cargo snapshot inputs come only from semantic target, dependency, workspace-dependency,
  target-specific dependency, patch, and replacement tables. An arbitrary metadata key named
  `path` is data, not filesystem authority. Semantic manifest paths are normalized relative to the
  declaring manifest; parent components and Windows-native backslashes are valid when the result
  remains beneath the retained root, while drive, UNC, rooted, traversal, symlink, and junction
  escapes remain rejected.
- Windows absolute-root containment parses native path components without lossy UTF-8 conversion
  and compares them with Win32 ordinal ignore-case semantics, including non-ASCII case variants.
- MCP issue verification requires explicit `GITHUB_TOKEN`, performs read/list/verify requests
  in-process without a provider subprocess, globally deduplicates/caps issue IDs, includes
  authentication/preflight in the complete deadline, and revalidates repository access after
  apparent absence. Discovery, bounded reads, and frontmatter parsing are part of the verification
  trust decision: an unreadable or malformed spec makes the operation inconclusive rather than
  disappearing from a successful zero-reference result.
- The latest review extends that same contract to checked recursive discovery and exact issue-field
  shapes through the shared maintained real-YAML parser. Duplicate/global malformed YAML and
  blank/null/wrong known shapes fail closed; comments/trailing commas remain valid; nested
  extension and block-scalar lookalikes are ignored. MCP diagnostic boundaries must not leak the
  server's absolute root, raw OS errors, or spec bytes: callers receive a sanitized relative path
  and a stable content-free reason.
- Windows confinement fixtures construct every path with native joins. The absolute-child read
  fixture is a valid one-file, fully covered project so its success proves root selection and
  downstream coverage execution; the junction fixture first proves that the reparse point targets
  the outside directory before asserting the intended confinement diagnostic and unchanged bytes.

## Files to Read First

- `src/mcp.rs`
- `src/generator.rs`
- `src/validator.rs`

## Current Status

Under CHG-0063 verification as a read-only-by-default agent-native MCP integration with exact
JSON-RPC envelope, argument, and resource validation; bounded capability snapshots and responses;
retained-capability, configuration-, Git-metadata-, and autodetection-level confinement; explicit
root-bound mutation; conservative unavailable-freshness scoring; atomic bounded generated-output
publication; and no embedded provider or credential surfaces. Shared real-YAML issue parsing,
checked top-level shapes, duplicate/global malformed rejection, checked traversal/non-UTF-8
discovery, and relative content-free diagnostics now have focused implementation and regression
coverage. Fresh definition reapproval, Windows runtime CI, independent rereview, and final
repository/trust/provenance/CI gates remain open.
