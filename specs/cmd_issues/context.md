---
spec: cmd_issues.spec.md
---

## Key Decisions

- Verification is delegated as one project-wide `github::verify_issue_batch`, which globally
  deduplicates/caps references and returns one `IssueVerification` per spec. This command gathers
  references, accumulates totals, formats output, and chooses the exit code.
- Closed issues are surfaced as warnings, not failures: only `not_found` (404) and `errors` drive the non-zero exit. This avoids breaking CI for legitimately-closed-but-still-referenced issues while still flagging them for review.
- `--create` preserves drift validation without reopening untrusted paths: the command snapshots
  mapped sources through retained capabilities and passes both retained spec bytes and
  `SourceSnapshot` observations to `validate_spec_content_with_sources`. Supplied-content export
  extraction never follows TypeScript wildcard imports through ambient paths.
- Specs without `implements` or `tracks` are skipped early so the command only talks to GitHub when there is something to verify.
- Repository/provider work happens after that scan. An empty project performs no Git auto-detection,
  credential use, or provider access, but an explicitly configured repository is still
  syntax-validated so malformed authority cannot hide behind a zero-reference, missing-specs, or
  empty-specs result.
- The human-readable summary branches on whether references were gathered, not on successful result
  counts, so provider-wide failures cannot masquerade as an empty project.
- Spec scanning is also part of the trust decision. Read failures and malformed/missing
  frontmatter become path-attributed inspection findings instead of being skipped. Diagnostics
  disclose neither bytes nor parser details; JSON uses stable `read_error` and
  `malformed_frontmatter` kinds, while Markdown/GitHub escape paths before table rendering.
- No-reference guidance is reserved for a complete, readable scan with no references. Any
  inspection finding appears in every supported format and forces exit 1 before a false-green
  empty result can be claimed.
- The final follow-up extends that same fail-closed decision to issue-field shape validation and
  recursive discovery. The shared maintained real-YAML parser rejects duplicate/global malformed
  YAML and blank/null/wrong-shaped known fields, accepts comments/trailing commas, and ignores
  nested extension/block-scalar lookalikes. A walker error cannot be flattened away as though no
  spec existed.
- CLI filesystem authority is capability-rooted from the project through the configured specs
  directory. One retained project-root capability authorizes both spec discovery and mapped-source
  snapshots, so replacing the ambient root path cannot split their authority. Child directories
  and files are opened with before/open/after identity checks, and bytes are read through the same
  verified file handle into immutable snapshots. The discovered identity remains binding through
  read completion, including regular-file and hardlink replacement races.
- Snapshot retention is explicitly bounded: at most 10,000 specs, 4 MiB per spec, and 64 MiB of
  retained spec bytes; the complete recursive inventory is capped at 100,000 entries including
  non-spec files. Mapped-source collection applies the same 4 MiB per-file and 64 MiB cumulative
  byte ceilings without ambient fallback.
- Diagnostic paths are data, not formatting. They remain project-relative and content-free,
  control, bidi-formatting, and Unicode line/paragraph separator characters are escaped for every
  renderer, and Markdown/GitHub table cells choose a safe code-span delimiter while escaping
  table-breaking characters. Code-span content is padded when a path begins or ends with a
  backtick so CommonMark does not misparse the edge delimiter.
- Relative diagnostic paths are normalized component-wise for display: Windows separators become
  `/`, while Unix keeps literal `\` bytes in filenames as escaped data.

## Files to Read First

- `src/commands/issues.rs` — the whole command, including format branches and the `--create` block.
- `src/github.rs` — `resolve_repo`, `verify_spec_issues`, and the `IssueVerification` / `GitHubIssue` types.
- `src/commands/mod.rs` — `build_schema_columns`, `create_drift_issues`.
- `src/parser.rs` — checked issue parsing plus compatibility frontmatter parsing for snapshot
  ownership collection.
- `src/validator.rs` — `SourceSnapshot` and `validate_spec_content_with_sources`, the exact
  spec-and-source snapshot validation seam.

## Current Status

CHG-0063 verification remains active. Provider classification, global deduplication/caps,
malformed output, transport failure, timeout behavior, bounded capability-rooted same-handle
spec/source snapshots, preserved snapshot-based `--create` validation, configured-repository
syntax checks, checked real-YAML issue fields, confined traversal, and renderer sanitization have
focused evidence. Fresh CHG definition reapproval, Windows runtime after the separator/fixture
repair, final independent rereview, trust/provenance, and
GitHub CI remain open; live provider success remains integration-only.

## Notes

- Output split: Text/Table/Csv share the human-readable path (per-spec detail, findings, and summary
  line); Json and Markdown/Github each have a structured path with explicit inspection findings.
- Part of the command layer — orchestrates the `github`, `validator`, and `commands` modules rather than containing domain logic.
