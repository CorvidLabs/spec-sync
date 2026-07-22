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
- `config`
- `github`
- `importer`
- `manifest`
- `mcp`
- `validator`

## Acceptance Criteria

- MCP rejects absolute outside roots before filesystem probing; uses retained directory capabilities and immutable bounded snapshots; parses Cargo and Gradle manifest-derived inputs without false-green omissions; never auto-detects issue repositories through project Git metadata; executes no provider subprocess for GitHub issue reads, listing, import, or verification and bounds globally deduplicated GitHub issue verification including selection, preflight, post-absence access revalidation, and typed fail-closed outcomes; GitHub batch imports strictly traverse at most 100 pages and fail on malformed pagination, duplicates, or cap truncation; rejects every case variant of .git configuration; validates complete JSON-RPC envelopes and resource arguments before dispatch; bounds actual project/config bytes and responses; preserves a bounded request ID or safely falls back to null; uses no-overwrite publication, preserves public replacements, retains ambiguous empty parents, and documents the same-user private-name threat boundary; reports all-error issue batches truthfully; Unix symlink and Windows junction tests prove confinement; all medium reviewer findings are closed; compatibility limits, full repository and trust gates, sandbox replay, Attest provenance, independent rereviews, and GitHub CI pass.

## No-spec Rationale

Not applicable
