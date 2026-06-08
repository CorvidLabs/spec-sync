---
spec: cmd_new.spec.md
---

## Tasks

- [ ] Add integration tests for `specsync new` CLI behavior (quick create, `--full`, refuse-overwrite, empty `files:` when no sources)

## Done

- [x] Implement `cmd_new` with source auto-detection (`detect_module_sources`) and export pre-population
- [x] Generate frontmatter + Purpose/Public API/Dependencies/Change Log skeleton
- [x] `--full` companion generation via `generator::generate_companion_files_for_spec`, with conditional `design.md`
- [x] Refuse to overwrite an existing spec (exit 1)
- [x] Replace unfinished-marker Public API rows with review prompts
- [x] In-module `chrono_lite_today()` date helper (no chrono dependency)

## Gaps

- No tests cover `cmd_new`; `src/commands/new.rs` has no `#[cfg(test)]` module and there are no `specsync new` integration tests.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
