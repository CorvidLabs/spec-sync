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
- Validate MCP selected config bytes and specs/source selector types before the compatibility
  loader can substitute defaults.

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
- Gather issue references before repository selection so an empty project never resolves a
  repository or contacts GitHub.
- Route missing-spec and repository-resolution outcomes through the selected JSON/Markdown/GitHub
  renderer before selecting the exit status.
