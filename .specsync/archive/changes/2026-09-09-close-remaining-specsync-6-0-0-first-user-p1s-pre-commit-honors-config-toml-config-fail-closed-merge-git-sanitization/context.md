---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: context
---

# Context

Independent proving after #770/#771 found three remaining first-user P1s on `main` (`7df304a`, which already includes #772 and #773). Do not revert the Action rc.14 pin. Do not touch #772/#773/#628.

## P1s closed here

1. First-user `init` → `add-spec` → `check` (0 errors, warnings) → `hooks install` → `git commit` fails because the generated pre-commit hook hardcodes `specsync check --strict`. Default 6.0 enforcement is strict on errors, not warnings. The hook must run `specsync check` so it honors `.specsync/config.toml`.
2. #653 is half-fixed: JSON parse-fail sets `load_error` and `load_config` refuses; TOML still goes through the silent line scanner, so `rules` / `rehash` / `compact` / `archive-tasks` / `deps` exit 0 over garbage. Choke point is `parse_config_content_checked` after reading TOML.
3. `merge::unmerged_paths` still spawns `Command::new("git")` with the full parent environment, so the MCP/check path via `cached_unmerged_paths` forwards `GITHUB_TOKEN`. It must use `git_cmd` (made `pub(crate)`).

Also in this package: workflow-v1 `Verifying` `next_action` names `verify` → `accept` → `archive` instead of v2 `check`/`review`/`finalize`; uncovered-path remediation uses `--kind bug-fix`; `MIGRATION.md` gets a copy-pasteable v1 close-out (F1/F4/F5/F6); confidence-report rows for #653 and MCP sanitization stop claiming a full fix.

## Constraints

- One change package, every touched `src/` path owned.
- Do not create tags, dispatch `promote`, `cargo publish`, touch Homebrew, force-push `main`, merge this PR, or close issues.
- Do not convert `change.rs` commit paths that rely on `GIT_AUTHOR_NAME`.
- Do not expand: fence-blindness, generate minting invalid specs, `fix_near_miss_required_headers`, nested-git half, interrupted-finalize attempts-ledger, CRLF.
