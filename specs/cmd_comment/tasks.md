---
spec: cmd_comment.spec.md
---

## Tasks

- [x] Add an end-to-end CLI test for stdout mode (`comment` without `--pr`) asserting the rendered markdown body on a fixture project — Evidence: `comment_reports_sdd_only_failures`, `comment_reports_sdd_failures_when_no_specs_exist`, and protocol-clean stdout coverage.

## Post-5.0 Roadmap

- [ ] Wire up the currently-unused `_base` parameter (diff base), or remove it if there is no planned use

## Done

- [x] Initial spec creation with all required sections
- [x] Requirements and acceptance criteria documented
- [x] Documented the unified pipeline: marketplace action + CI both use `specsync comment` for identical output
- [x] Verified the wrapper reuses `check`'s validation + `compute_exit_code` and renders via `comment::render_check_comment`
- [x] Confirmed the renderer is covered by `comment` inline tests (`test_render_check_comment_*`, `test_suggestion_for_*`)
- [x] Prevented configured SDD verification child output from contaminating comment stdout while preserving execution and failure reporting
- [x] Added end-to-end coverage proving comment mode is quiet while ordinary lifecycle checking still streams configured command output
- [x] Made project CI capture comment output with quiet Cargo and a defensive UTF-8-safe byte cap

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
