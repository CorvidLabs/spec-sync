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
- [x] Kept configured SDD verification output out of comment stdout. Since #543 `comment` executes no configured command, so nothing can reach it (REQ-cmd-comment-003, #848)
- [x] Added end-to-end coverage that neither `comment` nor `change check` spawns a configured command (`neither_comment_nor_change_check_spawns_configured_project_commands`)
- [x] Made project CI capture comment output with quiet Cargo and a defensive UTF-8-safe byte cap
- [x] Fail closed before rendering/posting when checked manifest discovery is inconclusive — Evidence: `malformed_gradle_is_inconclusive_for_coverage_gating_commands`.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
