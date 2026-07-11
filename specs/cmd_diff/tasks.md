---
spec: cmd_diff.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add a fixture that exercises the `--end-of-options` guard (base ref starting with `-`).
- [ ] Add a fixture for GitHub Actions PR auto-detection (`GITHUB_EVENT_NAME=pull_request` + `GITHUB_BASE_REF`).

## Done

- [x] `cmd_diff` implemented: cross-references `git diff --name-only` with spec `files:` lists and computes export deltas.
- [x] Hardened git invocation with `--end-of-options` to prevent argument injection from a `-`-prefixed base ref.
- [x] PR base auto-detection via `GITHUB_BASE_REF` when base is the default `HEAD`.
- [x] Detection of spec-file-only changes (`spec_modified` flag).
- [x] Integration coverage: `diff_shows_changes_since_base_ref`, `diff_no_changes_returns_empty`, `diff_detects_removed_exports`, `diff_human_readable_output`, `diff_detects_spec_file_only_changes`.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
