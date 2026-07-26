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
- `cli`
- `commands`
- `config`
- `exports`
- `generator`
- `github`
- `importer`
- `manifest`
- `mcp`
- `parser`
- `validator`

## Acceptance Criteria

- MCP rejects absolute outside roots before filesystem probing; uses retained directory capabilities and immutable bounded snapshots; accepts identity-bound original/canonical Windows startup spellings only to derive a suffix opened through the retained canonical capability and rejects sibling-prefix lookalikes; parses Cargo and Gradle manifest-derived inputs without false-green omissions; acquires selected config through no-follow, non-blocking, identity-verified regular-file snapshots and validates exact bounded bytes with the complete checked parser, rejecting non-object JSON, invalid UTF-8, malformed JSON/TOML, and wrong-typed known fields; never auto-detects issue repositories through project Git metadata; executes no provider subprocess for GitHub issue reads, listing, import, or verification and bounds globally deduplicated GitHub issue verification including selection, preflight, post-absence access revalidation, and typed fail-closed outcomes; GitHub batch imports strictly traverse at most 100 pages and fail on malformed pagination, duplicates, or cap truncation; rejects every case variant of .git configuration; validates complete JSON-RPC envelopes and resource arguments before dispatch; bounds actual project/config bytes and responses; preserves a bounded request ID or safely falls back to null; uses no-overwrite publication, preserves public replacements, retains ambiguous empty parents, and documents the same-user private-name threat boundary; reports all-error issue batches truthfully; CLI issue discovery binds discovered identity through read including regular/hardlink replacement, caps each spec and selected config at 4 MiB, retained spec/source snapshots at 64 MiB cumulatively, and spec count at 10,000, opens config/specs through one retained project capability, rejects linked/non-regular/replaced config and malformed or wrong-shaped exact config bytes, derives omitted source directories from a bounded sparse snapshot through that retained capability rather than the ambient root, validates malformed configured github.repo even with missing or empty specs, preserves structured JSON/Markdown/GitHub output for missing specs and repository failures, pads edge-backtick Markdown code spans, normalizes diagnostic separators only on Windows while preserving literal Unix backslashes as data, and validates issues --create through exact confined spec/source snapshots without ambient reopen or TypeScript wildcard resolution; drift terminal and GitHub issue output sanitizes hostile text; Unix symlink and Windows junction tests prove confinement; all medium reviewer findings are closed; compatibility limits, full repository and trust gates, sandbox replay, Attest provenance, independent rereviews, and GitHub CI pass.
- Selected MCP and CLI config/manifests are acquired with explicit no-follow, non-blocking retained handles whose opened metadata and identity remain authoritative on Windows and Unix; portable imported module names reject device aliases, trailing spaces/dots, and overlong generated filenames before writes; batch imports continue safely but exit nonzero after any item error; every promised JSON/PR-marker/replacement facet has a focused regression; and private-sandbox evidence binds the exact implementation binary plus drill/fixture bytes with immutable hashes.

## No-spec Rationale

Not applicable
