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
  buffers. Declared Cargo members and Node workspace patterns also consume a bounded expansion-work
  budget, while normalized completed workspace nodes are reused.
- Cargo snapshot inputs come only from semantic target, dependency, workspace-dependency,
  target-specific dependency, patch, and replacement tables. An arbitrary metadata key named
  `path` is data, not filesystem authority. Semantic manifest paths are normalized relative to the
  declaring manifest; parent components and Windows-native backslashes are valid when the result
  remains beneath the retained root, while drive, UNC, rooted, traversal, symlink, and junction
  escapes remain rejected.
- Windows absolute-root containment parses native path components without lossy UTF-8 conversion
  and compares them with Win32 ordinal ignore-case semantics, including non-ASCII case variants.
- Windows may canonicalize the requested temporary-directory root from an 8.3 spelling to its long
  spelling. Absolute read roots therefore derive a lexical suffix from either identity-bound
  startup spelling, but that suffix is always opened through the retained canonical capability;
  the ambient candidate path is never canonicalized or trusted as authority.
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
- Issue diagnostic path normalization is platform-specific: Windows separators render as `/`,
  while Unix literal backslashes remain filename bytes instead of being reinterpreted as hierarchy.
- Selected configuration and recognized manifests are acquired through explicit no-follow,
  non-blocking retained regular-file handles. Opened-handle metadata and identity are authoritative,
  with path-to-handle identity checks before and after bounded reads on Windows and Unix. The exact
  retained bytes pass complete checked parsing before compatibility
  loading, so non-object JSON, invalid UTF-8, malformed JSON/TOML, and wrong-typed known fields
  fail tools and resources instead of reverting to default paths.
- Every present Gradle build/settings candidate is preflighted through those retained handles with
  a 4 MiB per-file ceiling before any manifest-derived traversal, including candidates that would
  not otherwise be selected for parsing.
- Every present Gradle build/settings variant is preflighted through that reader at the shared
  4 MiB limit before settings parsing or source probing. The exact retained bytes are charged and
  copied once, and generic snapshot traversal skips those paths.
- Generic project-file traversal delegates to the same no-follow, non-blocking retained reader.
  Path metadata and opened-handle identity must remain equal before and after bounded reads on Unix
  and Windows, so FIFO/socket/device, link/reparse, and regular replacement races fail identically
  for tools and resources without consuming attacker bytes.
- Unix verification keeps FIFO behavior mandatory and executes socket behavior when the host
  permits creating a local socket fixture; restricted sandboxes that return `PermissionDenied`
  skip only that unavailable fixture instead of failing before the reader is exercised.
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
discovery, relative content-free diagnostics, Windows startup-alias absolute-child handling, and
selected-config fail-closed validation now have focused implementation and regression coverage.
The latest remediation charges Cargo/Node workspace declarations before deduplication, reuses
normalized completed nodes in snapshot and preflight traversal, and keeps zero-config discovery
capability-retained. Reported targeted runs passed 117 MCP unit tests and 65 MCP integration tests.
The full post-remediation suite passes 1,948 unit and 310 integration tests. Fresh exact-tree
independent rereview and hosted-Windows junction/reparse runtime evidence remain pending.
Earlier independent reviews additionally closed selected-config substitution, wrong-shaped
legacy GitHub fields, and blocking/special-file snapshot manifests. Configuration and manifest
acquisition now makes the no-follow opened regular-file handle authoritative, uses bounded
non-blocking reads, and rechecks path identity afterward. Generic project inputs now use the same
identity-continuous reader, with tool/resource race regressions for FIFO, symlink, and regular
replacements. An earlier
private-sandbox replay passed but its untracked inputs were not reproducible from the cited
revisions, so a hash-bound exact-tree replay is required with fresh definition approval, Windows
runtime CI, independent rereview, and final repository/trust/provenance/CI gates.
