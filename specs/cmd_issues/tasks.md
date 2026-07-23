---
spec: cmd_issues.spec.md
---

## Tasks

- [x] Reject malformed top-level `implements`/`tracks` shapes and entries through checked
  frontmatter inspection while ignoring nested extension keys and block-scalar text.
- [x] Use checked, confined recursive spec discovery so traversal failures, non-UTF-8 spec names,
  and escaping or symlinked `specs_dir` values cannot become an empty success.
- [x] Sanitize hostile finding paths and render Markdown/GitHub paths with valid escaped code spans.
- [x] Root CLI spec discovery in retained capabilities and read each immutable snapshot through the
  same identity-checked file handle.
- [x] Preserve `--create` validation against retained spec/source snapshots through
  `validate_spec_content_with_sources` without reopening paths or resolving ambient TypeScript
  wildcard imports.
- [x] Bind discovered identity through read completion, including regular-file and hardlink
  replacement, and enforce 4 MiB per-spec, 64 MiB cumulative, and 10,000-spec limits.
- [x] Validate configured repository syntax even when no references exist, without Git/provider
  access.
- [x] Escape bidi formatting controls and Unicode line/paragraph separators across renderers.
- [x] Pad Markdown/GitHub code-span content when hostile paths begin or end with backticks.
- [x] Normalize Windows finding paths to forward slashes without rewriting literal Unix filename
  backslashes.

## Post-5.0 Test Debt

- [x] Add command-level coverage for the no-references path with and without configured `github.repo`.
- [x] Use one retained project-root capability for spec discovery and mapped-source snapshots.
- [x] Bound the complete recursive inventory at 100,000 entries, including non-spec files.
- [ ] Add a mocked/recorded GitHub fixture to cover valid/closed/not-found classification and the non-zero exit on 404.

## Done

- [x] Verifies all `implements`/`tracks` references through one bounded globally deduplicated batch,
  tallying valid/closed/not-found/error counts per spec.
- [x] Repo resolution via `github::resolve_repo` with a clear error + exit 1 when unresolvable.
- [x] Text/Table/Csv, Json, and Markdown/Github output formats.
- [x] `--create` runs validation and opens drift issues for specs with errors.
- [x] Non-zero exit when any reference is not found or errored.
- [x] Gather references before repository/provider resolution and skip GitHub entirely when empty.
- [x] Add a command-level missing-token regression with per-spec JSON error attribution.
- [x] Retain unreadable and malformed specs as content-free findings in every supported format,
  suppress no-reference guidance, and exit 1
- [x] Share maintained real-YAML checked issue parsing with MCP, including duplicate/global
  malformed YAML rejection and valid comment/trailing-comma support.
- [x] Preserve valid issue references in CRLF specs through the shared checked parser.

## Gaps

- Network-free command fixtures cover the no-reference path; end-to-end provider classification
  still needs recorded fixtures or a mock process boundary.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
