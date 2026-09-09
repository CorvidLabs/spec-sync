---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: research
---

# Research

- `PRE_COMMIT_HOOK` hardcodes `specsync check --strict`; `pre_commit_block` rewrites that exact string to `specsync --root {} check --strict`. Tests at `install_precommit_creates_hook_file` and `install_precommit_uses_configured_hooks_path_and_preserves_exit_zero` assert `check --strict`. Default `enforcement` is already strict on errors; `--strict` additionally fails warnings. First-run `add-spec` emits ~13 warnings, so the hook is the first-user P1.
- `load_json_config` sets `load_error` on parse Err. `load_toml_config` calls `parse_toml_config`, a silent line scanner. `parse_config_content_checked` already uses `toml::from_str` + `validate_toml_config_types` and is what `check` uses, which is why `check` fails on garbage TOML while `rules`/`rehash`/`compact` do not. Invalid `enforcement` currently warns and keeps the default.
- `git_cmd` is private in `git_utils.rs`. `merge::unmerged_paths` uses `process::Command::new("git")` with no `env_clear`. MCP/`check` reach it via `cached_unmerged_paths`. `change.rs:7606` and other commit paths must keep `GIT_AUTHOR_NAME`; do not convert those.
- `summarize_change` Verifying arm always names v2 `check`/`review`/`finalize`. `scoped_review_current` is false for v1 (no scoped review file), so a v1 Verifying change is told to `change review`. Accepted already splits v2 finalize vs v1 archive.
- Uncovered-path remediation hardcodes `--kind fix`; clap expects `bug-fix`.
