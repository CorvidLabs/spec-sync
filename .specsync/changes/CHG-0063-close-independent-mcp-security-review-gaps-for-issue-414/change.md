---
id: CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414
state: implementing
type: feature
base_commit: a0d993b7d10d177f9a4770f54fbe14045750221c
---

# Close independent MCP security review gaps for issue 414

## Intent

Close independent MCP security review gaps for issue 414

## Affected Canonical Specs

- `cmd_check`
- `cmd_comment`
- `cmd_coverage`
- `cmd_generate`
- `cmd_import`
- `cmd_issues`
- `cmd_report`
- `cmd_score`
- `commands`
- `config`
- `exports`
- `github`
- `importer`
- `manifest`
- `mcp`
- `parser`
- `validator`

## Acceptance Criteria

- MCP rejects absolute outside roots before filesystem probing; uses retained directory capabilities and immutable bounded snapshots; accepts identity-bound original/canonical Windows startup spellings only to derive a suffix opened through the retained canonical capability and rejects sibling-prefix lookalikes; parses Cargo and Gradle manifest-derived inputs without false-green omissions; acquires selected config through no-follow, non-blocking, identity-verified regular-file snapshots and validates exact bounded bytes with the complete checked parser, rejecting non-object JSON, invalid UTF-8, malformed JSON/TOML, and wrong-typed known fields; never auto-detects issue repositories through project Git metadata; executes no provider subprocess for GitHub issue reads, listing, import, or verification and bounds globally deduplicated GitHub issue verification including selection, preflight, post-absence access revalidation, and typed fail-closed outcomes; GitHub batch imports strictly traverse at most 100 pages and fail on malformed pagination, duplicates, or cap truncation; rejects every case variant of .git configuration; validates complete JSON-RPC envelopes and resource arguments before dispatch; bounds actual project/config bytes and responses; preserves a bounded request ID or safely falls back to null; uses no-overwrite publication, preserves public replacements, retains ambiguous empty parents, and documents the same-user private-name threat boundary; reports all-error issue batches truthfully; CLI issue discovery binds discovered identity through read including regular/hardlink replacement, caps each spec and selected config at 4 MiB, retained spec/source snapshots at 64 MiB cumulatively, and spec count at 10,000, opens config/specs through one retained project capability, rejects linked/non-regular/replaced config and malformed or wrong-shaped exact config bytes, derives omitted source directories from a bounded sparse snapshot through that retained capability rather than the ambient root, validates malformed configured github.repo even with missing or empty specs, preserves structured JSON/Markdown/GitHub output for missing specs and repository failures, pads edge-backtick Markdown code spans, normalizes diagnostic separators only on Windows while preserving literal Unix backslashes as data, and validates issues --create through exact confined spec/source snapshots without ambient reopen or TypeScript wildcard resolution; drift terminal and GitHub issue output sanitizes hostile text; Unix symlink and Windows junction tests prove confinement; all medium reviewer findings are closed; compatibility limits, full repository and trust gates, sandbox replay, Attest provenance, independent rereviews, and GitHub CI pass.
- Selected MCP and CLI config/manifests are acquired with explicit no-follow, non-blocking retained handles whose opened metadata and identity remain authoritative on Windows and Unix; portable imported module names reject device aliases, trailing spaces/dots, and overlong generated filenames before writes; batch imports continue safely but exit nonzero after any item error; every promised JSON/PR-marker/replacement facet has a focused regression; and private-sandbox evidence binds the exact implementation binary plus drill/fixture bytes with immutable hashes.
- Shared Gradle discovery rejects raw drive-qualified include/project-selector identities before
  colon mapping; parses and confines only literal `setProjectDir(file(...))` and
  `setProjectDir(new File(rootDir, ...))` calls while dynamic or unsupported mutations fail closed;
  and checks every derived directory component no-follow through the retained project capability,
  rejecting Unix symlinks and Windows reparse points before CLI/MCP probing or traversal.
  Double-quoted interpolation, including encoded dollar spellings, is dynamic and rejects before
  discovery while escaped literal dollars and Groovy single-quoted literals remain compatible.
  Present Gradle build/settings manifests are bounded regular non-link files read through the
  retained capability. Fresh focused/full reruns, two independent reviews, hosted-Windows runtime,
  definition approval, repository/CI, trust, and Attest evidence are required for this amendment.
- Post-review closure preflights every present Gradle build/settings filename before precedence
  selection, binds each manifest's native identity before/opened/after its bounded read, rejects
  invoked unsupported inclusion APIs, and limits control-flow rejection to governed directives.
  CLI coverage uses one retained project capability for caller-selected spec ownership, every
  recognized manifest ecosystem, spec-module enumeration, and source discovery with iterative
  8 MiB/file, 64 MiB cumulative, 100,000-entry, and 256-component bounds plus strict UTF-8 and
  identity continuity. Every generic MCP project file uses a
  no-follow, non-blocking, identity-continuous retained read for tools and resources; FIFO/socket,
  link/reparse, and regular replacement races fail without attacker-byte consumption or partial
  output. Exact-head review remediation additionally retains the root before configuration and
  omitted-source detection, skips autodetection for explicit source roots, verifies nested
  config/manifest-directory reachability, carries selected-spec inventory identities into ownership
  parsing, charges selected-spec and source bytes plus entries to one checked-coverage budget, and
  bounds/deduplicates Cargo/Node expansion in both manifest and MCP traversal. Separate early and
  post-discovery checkpoints cover the checked-coverage operation and propagate failures to gate
  callers. A command-wide immutable CLI analysis snapshot and generic structured discovery outcomes
  remain assigned to later CLI/outcome/generation work outside issue #414's MCP boundary. The
  independent-rereview remediation additionally binds selected source-directory identities through
  coverage, revalidates selected-config parent chains after reads, and rejects authority-bearing
  recursive directory replacement. Exact-head rereview closure records sibling identities before
  sequential capability opens and releases completed Node bases/configured coverage roots so MCP
  and checked coverage remain descriptor-bounded across sibling and root breadth, consumes
  Node workspace child manifests/probes through identity-matching enumerated capabilities so
  swap/read/restore cannot mix generations, requires `workspaces.packages` for object-form Node
  workspaces, and strictly parses every recognized nested package manifest. Cargo operational
  discovery uses the same real-TOML multiline member semantics as preflight. The latest amended
  suite passes 1,953 unit and 312 integration tests. Both exact `237e548` rereviews rejected that
  candidate with three Medium findings. Both exact `971c89a` rereviews then rejected its incomplete
  descriptor bound across distinct Node bases and configured coverage roots. Its hash-bound
  private-sandbox replay is historical. The hash-bound exact `bead6d2` private-sandbox replay
  passes, and one independent rereview passed with zero High/Medium findings; the adversarial
  rereview rejected that candidate because a direct or nested mixed-case `.git` read root could
  become operation authority. The amended implementation rejects such selectors before opening
  the operation root. The amended tree passes 1,954 unit and 313 integration tests, release and
  Windows GNU cross-target compilation, strict 100% file/LOC coverage, all 62 scores at 100/A,
  documentation tests/lint/build, and editor-extension compile/package. Fresh exact-tree review,
  sandbox, hosted-Windows runtime, and final trust/provenance/CI evidence remain pending.

## No-spec Rationale

Not applicable
