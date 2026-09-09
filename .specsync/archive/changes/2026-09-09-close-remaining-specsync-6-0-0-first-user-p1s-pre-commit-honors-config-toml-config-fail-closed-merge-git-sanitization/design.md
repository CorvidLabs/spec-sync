---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: design
---

# Design

Local, fail-closed fixes. No new verbs.

- Generated pre-commit hook runs `specsync check` (and `specsync --root <project> check` in the managed block), not `check --strict`. Comment names `.specsync/config.toml` and the real default (strict on errors, warnings pass unless `--strict` or config says otherwise).
- `load_toml_config` parses through `parse_config_content_checked`. On `Err` or empty content it sets `load_error` with the same wording as JSON. `validate_toml_config_types` rejects an unknown `enforcement` enum instead of warning and keeping the default. Directory-as-config already fails via unreadable-file `load_error`.
- `git_cmd` is `pub(crate)`. `unmerged_paths` uses it and does not set `current_dir` again (`git_cmd` already does). Do not convert `change.rs` commit paths that need `GIT_AUTHOR_NAME`.
- `summarize_change` Verifying arm: if `workflow_version < 2`, next_action is `verify` then `accept` then `archive`. Leave the Accepted arm alone.
- Uncovered-path remediation uses `--kind bug-fix`.
- `MIGRATION.md` step 4 gets a literal v1 close-out: merge-then-archive, commit `.specsync/workflow-v2-baseline.json` and `.specsync/adoption-report.json` after adopt, names both `change check` and that `verify` no longer runs `verification_commands`, and states `adopt` is silent on a committed still-active v1.
