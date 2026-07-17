---
spec: github.spec.md
---

## Key Decisions

- **`gh` CLI first, REST fallback**: every read path (`fetch_issue`, `list_issues`) calls `gh_is_available` and prefers the CLI, falling back to direct `ureq` REST calls with `GITHUB_TOKEN`. Issue creation (`create_drift_issue`) is `gh`-only.
- **Token redaction**: `redact_token` strips any verbatim `GITHUB_TOKEN` occurrence from REST error strings before they surface (added 4.3.5). The token travels in the `Authorization` header, so this is defense-in-depth against a misbehaving proxy/redirect echoing it back.
- **State normalization**: issue `state` is lowercased (`"open"`/`"closed"`) so callers compare consistently regardless of CLI vs REST casing.
- **github.com only**: URL parsing handles `git@github.com:`, `https://github.com/`, and `http://github.com/`; GitHub Enterprise hosts are out of scope.
- **Deterministic hosted Bun runtime**: Pages, site CI, and VS Code extension CI use one exact Bun
  version and the expected `setup-bun` Action ref. `.github/scripts/validate-workflow-runtime-pins.py`
  validates every matching setup step and rejects moving refs, duplicates, unexpected jobs, or a
  missing nested `bun-version`, preventing a live tag-discovery dependency from returning. Action
  repository names are matched case-insensitively, as GitHub resolves them, while refs remain exact
  and case-sensitive so mixed-case owner/repository spellings cannot bypass the pin guard.
- **Monotonic Action promotion**: immutable `v<major>.<minor>.<patch>` refs are verified before the
  compatible floating `v<major>` ref advances. Release metadata remains synchronized through
  `.github/scripts/validate-release-version.py`, which rejects every README/site Action ref other
  than the exact candidate ref, including moving branch names such as `main`. It parses fenced YAML
  through Psych, including backtick or tilde fences with up to three leading spaces and metadata
  such as `title="ci.yml"`, so named/nested `uses` steps and block or flow `with.version` mappings
  are covered without mistaking cross-project reference prose for an Action step. Workflow
  validation also recognizes quoted `uses` keys and rejects flow-style setup-bun steps that evade
  the maintained block-style pin contract. Action repository names are normalized case-insensitively,
  while the selected release ref is compared exactly.
- **Hermetic release guards**: release and runtime-pin validators require no Python site packages;
  the release guard uses Ruby's standard-library Psych parser for full YAML syntax validation.
  Cargo metadata is read without Python 3.11-only `tomllib`, lifecycle verification declares the
  Ruby preflight explicitly, and hosted CI provisions a pinned Ruby runtime, so Python 3.10+
  verification depends on neither ambient PyYAML nor an undeclared hosted runtime.

## Key Files

- `src/github.rs` - Main implementation: repo detection, `gh`/REST issue fetch, `list_issues`, `create_drift_issue`, `redact_token`
- `src/commands/mod.rs` - `create_drift_issues` wires `github.drift_labels` (default `["spec-drift"]`) into `create_drift_issue`
- `specs/github/github.spec.md` - Module specification
- `specs/github/requirements.md` - User stories and acceptance criteria
- `.github/scripts/validate-release-version.py` - Current package, Action, docs, CI consumer, and
  Trust candidate version consistency
- `.github/scripts/validate-workflow-runtime-pins.py` - Exact hosted Bun runtime enforcement

## Current Status

Module behavior is stable. URL parsing is unit-tested; network paths remain manual/integration
coverage. The 5.1.1 release candidate adds deterministic Action/runtime distribution checks, while
external exact/floating ref smoke tests remain publication-time gates.
