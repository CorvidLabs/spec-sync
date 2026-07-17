---
spec: github.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/github.rs` | cargo test github:: | `test_parse_repo_from_url_https`, `test_parse_repo_from_url_ssh`, `test_parse_repo_from_url_unknown` |
| Release version surfaces | `ruby --version` then `PYENV_VERSION=3.10.20 python3 -S .github/scripts/validate-release-version.py` | Pinned hosted Ruby and the declared lifecycle preflight provide Psych; maintained YAML syntax, Cargo, lockfile, Action default, every README/site YAML Action step (including indented or metadata-bearing fences, named/nested `uses`, mixed-case repositories, non-version moving refs, and block/flow `with.version` inputs), packaged consumer, Trust candidate, checkout contract, and changelog agree without Python site packages or Python 3.11-only modules |
| Hosted Bun runtime | `python3 -S .github/scripts/validate-workflow-runtime-pins.py` | Pages, site CI, and VS Code extension CI each contain exactly one expected `setup-bun` Action ref with the supported exact Bun version under that step's `with` mapping; quoted `uses` keys, mixed-case repositories, moving refs, duplicates, unexpected jobs, and missing nested inputs fail without Python site packages |

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
| GitHub Bun tag API unavailable | Maintained JS jobs do not perform latest-tag discovery | Keep exact `bun-version` pins in all three `setup-bun` jobs and run their frozen install/build commands |
| Mixed-case Action repository spelling | Repository matching follows GitHub's case-insensitive resolution, but refs remain exact and case-sensitive | Exercise mixed-case `setup-bun` and `checkout` repository spellings with a disallowed ref and require the applicable validator to fail |
| YAML fence includes metadata | The fenced workflow example remains in release validation scope | Add fence metadata such as `title="ci.yml"`, substitute a moving spec-sync ref, and require the release validator to fail |
| YAML fence is indented | A CommonMark fence with one to three leading spaces remains in release validation scope | Indent a YAML fence, substitute a moving spec-sync ref, and require the release validator to fail |
| Workflow `uses` key is quoted | The YAML-equivalent step remains in runtime-pin validation scope | Add a quoted `"uses"` key with a moving setup-bun ref and require the runtime validator to fail |
| Exact Action release fails a platform smoke test | Floating `v5` remains unchanged | Compare refs before and after the failed promotion attempt |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/github.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
- Before promoting an Action release, run pinned `@v<major>.<minor>.<patch>` consumers on Linux,
  macOS, and Windows; advance `v<major>` only after all pass.
