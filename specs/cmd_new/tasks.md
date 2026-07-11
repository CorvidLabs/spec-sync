---
spec: cmd_new.spec.md
---

## Tasks

- [x] Add integration coverage for basic `specsync new` creation, source auto-detection, no-match guidance, and module-name safety — Evidence: `new_auto_detects_single_source_file`, `new_warns_when_no_source_files_match`, and traversal rejection.

## Post-5.0 Test Debt

- [ ] Add explicit integration coverage for `specsync new --full` and refuse-overwrite behavior.

## Done

- [x] Implement `cmd_new` with source auto-detection (`detect_module_sources`) and export pre-population
- [x] Generate frontmatter + Purpose/Public API/Dependencies/Change Log skeleton
- [x] `--full` companion generation via `generator::generate_companion_files_for_spec`, with conditional `design.md`
- [x] Refuse to overwrite an existing spec (exit 1)
- [x] Replace unfinished-marker Public API rows with review prompts
- [x] In-module `chrono_lite_today()` date helper (no chrono dependency)

## Gaps

- No tests cover `cmd_new`; `src/commands/new.rs` has no `#[cfg(test)]` module and there are no `specsync new` integration tests.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
