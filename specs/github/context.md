---
spec: github.spec.md
---

## Key Decisions

- **`gh` CLI first, REST fallback**: every read path (`fetch_issue`, `list_issues`) calls `gh_is_available` and prefers the CLI, falling back to direct `ureq` REST calls with `GITHUB_TOKEN`. Issue creation (`create_drift_issue`) is `gh`-only.
- **Token redaction**: `redact_token` strips any verbatim `GITHUB_TOKEN` occurrence from REST error strings before they surface (added 4.3.5). The token travels in the `Authorization` header, so this is defense-in-depth against a misbehaving proxy/redirect echoing it back.
- **State normalization**: issue `state` is lowercased (`"open"`/`"closed"`) so callers compare consistently regardless of CLI vs REST casing.
- **github.com only**: URL parsing handles `git@github.com:`, `https://github.com/`, and `http://github.com/`; GitHub Enterprise hosts are out of scope.

## Key Files

- `src/github.rs` - Main implementation: repo detection, `gh`/REST issue fetch, `list_issues`, `create_drift_issue`, `redact_token`
- `src/commands/mod.rs` - `create_drift_issues` wires `github.drift_labels` (default `["spec-drift"]`) into `create_drift_issue`
- `specs/github/github.spec.md` - Module specification
- `specs/github/requirements.md` - User stories and acceptance criteria

## Current Status

Module is stable and complete. Only URL parsing is unit-tested; network paths are exercised manually / in integration. All requirements documented.
