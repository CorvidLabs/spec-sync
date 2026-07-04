---
spec: cmd_diff.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/diff.rs` | cargo test --test integration diff_ | No inline `#[cfg(test)]` module in `diff.rs`; behavior is covered end-to-end. Add inline coverage for `detect_pr_base` and the `--end-of-options` guard before risky changes. |
| `tests/integration.rs` | cargo test --test integration diff_shows_changes_since_base_ref | Staged new export `logout` appears in `changes[0].new_exports` (JSON output). |
| `tests/integration.rs` | cargo test --test integration diff_no_changes_returns_empty | Clean tree against `HEAD` yields `{"changes":[]}`. |
| `tests/integration.rs` | cargo test --test integration diff_bad_base_ref_fails_loud | A bogus `--base` ref exits non-zero, names the ref on stderr, and never prints "No files changed" (fail-loud, not fail-open). |
| `tests/integration.rs` | cargo test --test integration diff_detects_removed_exports | Removed export `logout` (still in spec) appears in `changes[0].removed_exports`. |
| `tests/integration.rs` | cargo test --test integration diff_human_readable_output | Default Text format prints the `auth` spec and new export `signup`. |
| `tests/integration.rs` | cargo test --test integration diff_detects_spec_file_only_changes | Spec-only edit reports one change with `spec_modified == true` and empty `changed_files`. |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| New export added | `src/auth/service.ts` gains `export function signup()` after the base commit | `specsync diff --base HEAD` (Text) | Prints the `auth` spec and `+ New exports (not in spec): signup`. |
| New export added (JSON) | same as above | `specsync diff --base HEAD --json` | `changes[0].new_exports` contains the new symbol. |
| Spec-only change | only the `.spec.md` is edited; no tracked source files change | `specsync diff --base HEAD --json` | One entry with `spec_modified: true` and empty `changed_files`. |
| No changes | nothing changed since base | `specsync diff` | JSON: `{"changes":[]}`; Markdown: "No files changed since `<base>`."; Text: "No files changed since <base>". |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| `git` fails to spawn | Prints "Failed to run git diff" to stderr and exits 1 | Keep; not currently asserted by a fixture — add before changing the spawn-error path. |
| Bad/unknown base ref | `git diff` exits non-zero (empty stdout); command inspects the exit status and fails loud (exit 1, git's stderr surfaced, names the base ref) instead of reporting "no changes" — so a failed comparison cannot silently pass in CI | Asserted by `diff_bad_base_ref_fails_loud`; keep the status check before touching the git invocation. |
| Base ref starting with `-` | Parsed as a revision via `--end-of-options`, never as a git flag | Add a fixture before touching the `git diff` argument list. |
| Changed source file not in any spec (Text format) | Listed under "Changed files not covered by any spec" when no specs matched | Keep or add a focused assertion before changing this behavior. |
| PR context (`pull_request` + `GITHUB_BASE_REF`) and default `HEAD` base | Compares against `origin/<base_ref>` and logs the detection to stderr | Add a fixture that sets the env vars before changing `detect_pr_base`. |

## Reviewer Checklist

- Run `cargo run -- diff --help` and confirm the help text still names the documented flags (`--base`, `--format`/`--json`).
- Run `cargo test --test integration diff_` before the full suite when changing `src/commands/diff.rs`.
- Reproduce one Behavioral Verification row with a temporary git fixture before changing user-visible output strings.
- If an output or error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
