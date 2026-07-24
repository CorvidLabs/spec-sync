---
change: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
artifact: design
---

# Design

## Authorization ordering

- Reject absolute requested roots that are not lexical descendants before calling canonicalize.
- Open and identity-bind the requested root before canonicalization, retain that initial
  `cap-std::fs::Dir`, reopen the canonical path through its parent capability, and require both
  handles to identify the same directory before serving requests.
- Resolve writes only through the retained initial capability and copy reads into an
  operation-scoped bounded snapshot.
- Canonicalize in-root candidates to catch symlink/junction escapes before snapshot traversal.
- Do not execute Git for MCP repository discovery; use only explicit `github.repo`.
- Reject every case variant of `.git` in configured roots and omit Git metadata from snapshots.
- For CLI issues, retain one project capability for config and specs. Open the selected config as
  a non-link regular file, bind its discovered/open/read-complete identity, cap it at 4 MiB, and
  parse/apply only the retained bytes.
- If CLI issue config omits source directories, build a bounded sparse detection snapshot through
  that retained project capability and pass the detected list into exact-byte config parsing.
- Open MCP selected config non-blocking through verified regular-directory and regular-file
  capabilities, reject symlink/reparse and special-file paths, require the opened identity to
  equal the pre-open inspected identity before reading, recheck after the bounded read, and pass
  the exact retained bytes through complete checked config parsing before the compatibility loader
  can substitute defaults.
- Open recognized snapshot manifests non-blocking, reject non-regular entries before parsing, and
  bind the opened identity to the inspected identity so replacement or special-file inputs cannot
  block or contribute unverified bytes.
- Apply the shared source-detection ignore policy before retained metadata inspection; a recognized
  non-regular manifest is an explicit configuration finding rather than an omitted source tree.

## Bounded I/O

- Accept at most 1 MiB per JSON-RPC input line and output response.
- Accept at most 8 MiB per project file and 64 MiB of actual project/config bytes per operation.
- Honor explicitly configured roots even when their names are normally ignored.
- Normalize root-wide inputs and discover manifest-derived workspace roots before applying fixed
  snapshot ignores. Parse Cargo as bounded TOML so comments/strings cannot impersonate table
  headers; parse multiline workspace arrays and charge normalized, deduplicated
  manifest bytes to the same cumulative budget before parsing; copy the exact preflighted manifest
  bytes rather than rereading a mutable path.
- Parse Gradle Groovy/Kotlin include syntax, comments, escapes, and supported `projectDir` overrides
  once in the manifest module, then use the resulting effective paths for both discovery and MCP
  confinement. Any malformed declaration fails the entire discovery result.
- Replace an oversized response with a compact `-32603` error, preserve only an ID that fits the
  response bound, fall back to `null` otherwise, and propagate transport failures.

## Protocol and mutation safety

- Validate request object, protocol version, method, ID, and params before notification suppression
  or method dispatch. Valid notifications remain silent and non-mutating.
- Validate `resources/read` against an exact `{uri: string}` schema.
- Bound generation to 1,000 specs and 64 MiB, preflight the serialized result, stage and sync each
  file beside its destination, retain its identity, publish only the verified staged identity, and
  fail without overwrite. Rollback atomically quarantines each public transaction pathname before
  comparing identity and preserves mismatches. Do not claim ownership of newly created parent
  directories across the non-atomic create/open interval; failed batches may retain empty parents.
  Private stage/quarantine names are isolated transaction internals, and a same-user process with
  independent write access to the root must not race them.
- Report Git freshness as unavailable in snapshots and withhold the freshness score instead of
  accidentally inspecting an enclosing checkout.
- Require explicit `GITHUB_TOKEN`, preflight configured GitHub repository access once, fetch at
  most 100 globally deduplicated issue IDs through in-process REST, revalidate access after an
  absent-issue response, strictly validate responses, and enforce authentication, preflight,
  operation, and whole-batch deadlines without spawning a provider subprocess.
- Reject pull-request markers from direct issue-detail responses and reject imported issue titles
  whose safe module slug is empty before constructing output.
- Gather issue references before repository selection so an empty project never resolves a
  repository or contacts GitHub.
- Route missing-spec and repository-resolution outcomes through the selected JSON/Markdown/GitHub
  renderer before selecting the exit status.

## Final retained-handle and import amendment

- Treat the first no-follow, non-blocking regular-file handle as authority. Compare all later path
  observations to that handle identity before and after bounded reads, including on Windows.
- Apply one portable module-name validator before every import output join. Batch orchestration
  returns structured counts so the outer command can continue items yet exit 1 after any error.
- Make evidence reproducible by building from the cited implementation commit and hashing the
  executable plus every drill/fixture input used by the private sandbox.

## Gradle discovery authority amendment

- Validate each raw `include` identity and raw `project(...)` selector before replacing Gradle
  colon separators with filesystem separators. Reject drive-qualified, rooted, UNC, and
  parent-escaping spellings while preserving ordinary nested identities such as `:service:api`.
- Treat assignment-style `.projectDir = ...` and method-style `.setProjectDir(...)` as the same
  security boundary. Accept exactly `file(<literal>)` and `new File(rootDir, <literal>)`; reject
  variables, interpolation, alternate bases, extra arguments, trailing expressions, and
  unsupported project-directory mutators before returning any module.
- Retain one project-root directory capability for Gradle discovery. Resolve every component of
  each effective module directory with no-follow metadata/open semantics before source probing,
  reject Unix symlinks and Windows reparse points at any depth, and never hand an unchecked
  ambient path to CLI coverage or MCP snapshot traversal.
- Preserve fail-closed compatibility: checked discovery returns the confinement/parser error;
  compatibility discovery may return empty, but every CLI/MCP gate must use the checked path and
  report an inconclusive non-success outcome.
- Parse Gradle string escapes before path authorization. Reject every unescaped dollar in
  double-quoted literals, including dollars produced by Unicode or octal decoding; decode an
  explicit `\$` as a literal dollar and preserve single-quoted Groovy dollar literals.
- Select and read present Gradle build/settings files through the retained root capability. Require
  regular non-link entries, cap each file at 4 MiB, parse the exact opened bytes, and fail checked
  discovery on links, reparse points, special files, invalid UTF-8, or replacement/type changes.

## Post-review authority closure

- Preflight every present Gradle build/settings filename before precedence selection. Bind the
  native pathname identity to the opened retained handle before open, after open, and after the
  bounded read on Unix and Windows.
- Reject invoked unsupported inclusion APIs and governed indirect/conditional mutations, but keep
  unrelated top-level Gradle control flow and identifier/documentation uses compatible.
- Retain one CLI project capability across manifest discovery, spec-module enumeration, iterative
  source traversal, and final root verification. Enforce 8 MiB per file, 64 MiB cumulative bytes,
  100,000 entries, 256 path components, strict UTF-8, and directory/file/root identity continuity.
- Route generic MCP project inputs through the same no-follow, non-blocking, identity-continuous
  retained reader used for selected config and manifests. Apply it to both tools and resources so
  FIFO/socket, link/reparse, and regular replacement races fail without partial output.
