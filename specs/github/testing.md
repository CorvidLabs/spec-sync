---
spec: github.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/github.rs` | cargo test github:: | `test_parse_repo_from_url_https`, `test_parse_repo_from_url_ssh`, `test_parse_repo_from_url_unknown` |

## Coverage Gaps

- Integration gap: add a fixture for "Verify spec issues" before changing user-visible CLI output, generated files, or error handling in github.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Verify spec issues | a spec with `implements: [42]` and `tracks: [100]`, issue #42 is open, #100 is closed | `verify_spec_issues` is called | returns `valid: [#42]`, `closed: [#100]`, `not_found: []`, `errors: []` |
| Auto-detect repo from SSH remote | git remote URL is `git@github.com:CorvidLabs/spec-sync.git` | `detect_repo(root)` is called | returns `Some("CorvidLabs/spec-sync")` |
| Create drift issue | a spec has validation errors | `create_drift_issue(repo, path, errors, labels)` is called | creates a GitHub issue titled "Spec drift detected: {path}" with error list in body |
| gh CLI unavailable, API fallback | `gh auth status` fails but `GITHUB_TOKEN` is set | `fetch_issue(repo, 42)` is called | falls back to REST API and returns the issue |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No git remote configured | `detect_repo` returns `None` | Keep or add a focused assertion before changing this behavior |
| Neither config repo nor git remote | `resolve_repo` returns `Err` | Keep or add a focused assertion before changing this behavior |
| `gh` unavailable and no `GITHUB_TOKEN` | `fetch_issue` returns `Err` | Keep or add a focused assertion before changing this behavior |
| Issue does not exist (404) | `fetch_issue_api` returns `Err("Issue not found")` | Keep or add a focused assertion before changing this behavior |
| Network timeout | `fetch_issue_api` returns `Err` after 10 seconds | Keep or add a focused assertion before changing this behavior |
| `gh` CLI not authenticated | `gh_is_available` returns `false` | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/github.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
