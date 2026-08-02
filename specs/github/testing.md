---
spec: github.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/github.rs` | cargo test github:: | Focused source coverage includes URL parsing, injected prepare/fetch orchestration, per-spec attribution, cap-before-provider, full deadlines, post-404 access revalidation, strict single/list response parsing, raw-page bounds, validation of every issue/PR item before filtering, open-only state, exact URL identity, raw duplicate rejection, requested-repository/query-bound Link pagination, malformed/transport errors, and no-provider-subprocess guards. |
| Release version surfaces | `ruby --version` then `PYENV_VERSION=3.10.20 python3 -S .github/scripts/validate-release-version.py` | Pinned hosted Ruby and the declared lifecycle preflight provide Psych; maintained YAML syntax, Cargo, lockfile, Action default, every current root/site YAML Action step (including case-insensitive, indented, metadata-bearing, backtick, or tilde fences), structurally parsed block/flow workflow steps, named/nested/quoted keys, mixed-case repositories, non-version moving refs, and block/flow `with` inputs, packaged consumer, Trust candidate, checkout contract, and changelog agree without Python site packages or Python 3.11-only modules |
| Hosted Bun runtime | `python3 -S .github/scripts/validate-workflow-runtime-pins.py` | Pages, site CI, and VS Code extension CI each contain exactly one expected `setup-bun` Action ref with the supported exact Bun version under that step's `with` mapping; structural parsing covers block/flow mappings, quoted keys, arbitrary valid list-marker spacing, and `uses` after other keys, while mixed-case repositories, moving refs, duplicates, unexpected jobs, and missing inputs fail without Python site packages |
| Immutable RC evidence | `python3 .github/scripts/test-validate-release-candidate.py` | Exactly one successful Ubuntu, macOS, and Windows record must share the expected annotated RC identity, candidate SHA, schema, and Fledge lane; missing, duplicate, malformed, failed, cancelled, or mixed identity evidence fails closed |
| Metadata-descendant provenance | `python3 .github/scripts/test-reuse-check-from-ancestors.py` and `python3 .github/scripts/test-verify-trusted-policy-check.py` | First-parent bounds, second-parent exclusion, exact historical scoped-review and parent-bound workflow-v2 archive edges, native enum/timestamp/empty-manifest parity, metadata-check skipping, terminal product boundaries, exact check/job/run identity, complete check lookup, foreign and malformed rejection, unsuccessful-only failure, and success preference over newer cancellation/failure |

## Coverage Gaps

- Integration gap: add a fixture for "Verify spec issues" before changing user-visible CLI output, generated files, or error handling in github.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Verify spec issues | a spec with `implements: [42]` and `tracks: [100]`, issue #42 is open, #100 is closed | `verify_spec_issues` is called | returns `valid: [#42]`, `closed: [#100]`, `not_found: []`, `errors: []` |
| Auto-detect repo from SSH remote | git remote URL is `git@github.com:CorvidLabs/spec-sync.git` | `detect_repo(root)` is called | returns `Some("CorvidLabs/spec-sync")` |
| Create drift issue | a spec has validation errors | `create_drift_issue(repo, path, errors, labels)` is called | creates a GitHub issue titled "Spec drift detected: {path}" with error list in body |
| Authenticated gh without REST token | `gh auth status` succeeds but `GITHUB_TOKEN` is unset | `fetch_issue(repo, 42)` is called | returns a token-required error and does not launch `gh issue view` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No git remote configured | `detect_repo` returns `None` | Keep or add a focused assertion before changing this behavior |
| Neither config repo nor git remote | `resolve_repo` returns `Err` | Keep or add a focused assertion before changing this behavior |
| No `GITHUB_TOKEN` | Read/list/verify paths fail without consulting authenticated `gh` state | Keep the legacy-read rejection and batch-prepare authentication assertions |
| Read/list/verify/import subprocess boundary | All platforms reject `gh` construction in read modules; on Unix, every token-present entry also fails through an isolated unreachable local REST endpoint without executing a PATH-injected sentinel | Covered by `provider_process_construction_is_absent_from_every_read_path` and `token_present_read_list_verify_and_import_paths_never_spawn_gh` |
| Issue does not exist (404) | `fetch_issue_api` returns `Err("Issue not found")` | Keep or add a focused assertion before changing this behavior |
| Network timeout | `fetch_issue_api` returns `Err` after 10 seconds | Keep or add a focused assertion before changing this behavior |
| `gh` CLI not authenticated | `gh_is_available` returns `false` | Keep or add a focused assertion before changing this behavior |
| Inaccessible/private repository | Inconclusive provider error, not issue not_found | Exercise the repository-vs-issue typed classification |
| Access revoked after issue 404 | Repository recheck fails inside the remaining operation/batch time and the result is inconclusive | Exercise recheck-failure classification separately from confirmed absence |
| Duplicate IDs across specs | One provider lookup per unique ID | Exercise global batch deduplication |
| More than 100 unique IDs | Fail before provider selection | Exercise the invocation cap through MCP |
| Malformed or unavailable REST provider | Bounded inconclusive error | Exercise strict response parsing, transport failure, operation deadline, and full-batch deadline |
| Malformed issue-list item or pull-request marker, including explicit `null` | Entire listing fails closed; no default number/title/state is synthesized and null is not treated as an issue | Covered by `issue_list_requires_present_pull_request_marker_to_be_an_object` |
| Closed raw issue or pull request | Entire listing fails before PR filtering because `state=open` is an endpoint invariant | Covered by `issue_list_requires_exact_raw_open_state_for_issues_and_pull_requests` |
| Raw issue/PR URL uses the wrong repo, resource kind, or number | Entire listing fails before PR filtering | Covered by `issue_list_rejects_wrong_issue_and_pull_request_url_identity` |
| Raw issue/PR URL number has leading zeros | Entire listing fails before PR filtering even when the numeric value matches | Covered by `provider_item_urls_require_canonical_decimal_numbers_in_list_and_detail` |
| Duplicate raw identity involves one or more pull requests | Entire page/pagination traversal fails before filtering can hide the collision | Covered by the raw duplicate regressions within and across pages |
| Direct issue-detail payload carries object, null, or scalar `pull_request` | Reject every marker shape as provider data for a pull request; do not verify or import it as an issue | Covered by `issue_details_reject_pull_request_markers_of_any_shape` |
| Issue-list page has exactly 100 provider entries, including a pull request | Page is accepted and the pull request is filtered only after the page bound is established | Covered by `issue_list_accepts_one_hundred_provider_entries_including_pull_requests` |
| Issue-list page has 101 provider entries | Entire listing fails before parsing the malformed overflow item | Covered by `issue_list_rejects_one_hundred_one_entries_before_parsing_malformed_pull_request` |
| Duplicate issue across pages or next link after page 100 | Entire listing fails instead of deduplicating or truncating | Covered by `issue_list_pagination_fails_instead_of_truncating_or_deduplicating` |
| Malformed `Link` header | Entire listing fails before another request | Covered by `link_header_parsing_detects_next_and_rejects_malformed_values` |
| Next link targets another repository or resource | Entire listing fails before another request | Covered by `link_header_rejects_wrong_or_malformed_repository_identity_and_resource` |
| Next link changes open-state, page size, label, or page semantics | Entire listing fails before another request | Covered by `link_header_rejects_query_mismatch` |
| GitHub Bun tag API unavailable | Maintained JS jobs do not perform latest-tag discovery | Keep exact `bun-version` pins in all three `setup-bun` jobs and run their frozen install/build commands |
| Mixed-case Action repository spelling | Repository matching follows GitHub's case-insensitive resolution, but refs remain exact and case-sensitive | Exercise mixed-case `setup-bun` and `checkout` repository spellings with a disallowed ref and require the applicable validator to fail |
| YAML fence includes metadata | The fenced workflow example remains in release validation scope | Add fence metadata such as `title="ci.yml"`, substitute a moving spec-sync ref, and require the release validator to fail |
| YAML fence is indented | A CommonMark fence with one to three leading spaces remains in release validation scope | Indent a YAML fence, substitute a moving spec-sync ref, and require the release validator to fail |
| YAML fence separates its language with whitespace | A rendered YAML/YML example remains in release validation scope | Add horizontal whitespace between the fence marker and language, substitute a moving spec-sync ref, and require the release validator to fail |
| YAML fence uses a longer valid closer | A rendered YAML/YML example remains in release validation scope | Open with a four-character fence, close with five matching characters, substitute a moving spec-sync ref, and require the release validator to fail |
| YAML fence is unclosed | The CommonMark block extends to end-of-document and remains in release validation scope | Omit the closing fence, substitute a moving spec-sync ref, and require the release validator to fail |
| Root security guidance changes | CI runs the Action documentation validator | Change `SECURITY.md` alone and confirm the workflow path filters schedule `validate-action` |
| Root security guidance uses a moving inline ref | The inline recommendation remains pinned to the current floating major | Replace the inline `@v5` ref with `@main` and require the release validator to fail |
| Candidate mirror input is removed | Release CI continues to exercise the just-built candidate rather than a published binary | Remove either runner-local mirror input from the packaged Action consumer or Trust gate and require the release validator to fail |
| Trust lifecycle points at the full local suite | Hosted Trust duplicates `cargo test` already owned by CI | Parse `.trust.toml` and `fledge.toml`; require `trust-lifecycle`, reject `test`/`lint`/`verify` in that lane, and require the full `verify` lane to remain intact |
| Release workflow changes | CI runs the full validation path and release validator scans its Action steps | Change `.github/workflows/release.yml` alone and confirm CI is scheduled; use a disallowed spec-sync ref to require the validator to fail |
| Workflow `uses` key is quoted | The YAML-equivalent step remains in runtime-pin validation scope | Add a quoted `"uses"` key with a moving setup-bun ref and require the runtime validator to fail |
| Workflow checkout `uses` key is quoted | The YAML-equivalent checkout remains in release validation scope | Add a quoted `"uses"` key for an extra checkout step and require the release validator to fail |
| Workflow setup-bun step uses flow mapping syntax | The YAML-equivalent setup-bun step remains in runtime-pin validation scope | Add `{ uses: oven-sh/setup-bun@main }` and require the runtime validator to fail |
| YAML fence uses tildes | The rendered workflow example remains in release validation scope | Convert a public YAML example to a tilde fence, substitute a moving spec-sync ref, and require the release validator to fail |
| Exact Action release fails a platform smoke test | Floating `v5` remains unchanged | Compare refs before and after the failed promotion attempt |
| Ordinary product PR | Ubuntu owns the integration test job; macOS and Windows are not scheduled | Parse `.github/workflows/ci.yml` and require one Ubuntu test authority without an OS matrix |
| Immutable RC qualification | Ubuntu, macOS, and Windows run `fledge lanes run release-candidate` for the same resolved SHA | Parse the release matrix and validate fixture evidence for all three platforms |
| Candidate content or marker changes | Prior platform evidence cannot authorize promotion or upload | Change the expected SHA/tag in validator fixtures and require failure; conflicting workflow history also fails |
| Final publication | Final tag and artifacts use the already-qualified candidate SHA | Require authorization before promotion and independent final-tag/checkout identity checks before upload |
| Review/archive child follows a green product tip | Reuse required checks without rerunning the product matrix | Require one shared product ancestor and CI run; reject foreign app/PR/repository/workflow/SHA evidence |
| Intervening child changes product code | Older green evidence cannot authorize the new tip | Stop before the non-review/non-archive edge and fail closed |
| Newer exact-SHA policy result is cancelled after a success | Cancellation from moved-tip republication does not erase prior authenticated success | Keep the successful and unsuccessful-only fixtures in `test-verify-trusted-policy-check.py` |
| A second workflow-v2 change finalizes after an earlier archive child | Traverse the earlier exact archive edge to the shared product boundary | Require matching archived state plus finalization parent commit/tree and reject bad bindings |
| A reusable Actions check points only at a run or at another job/check | Reject the candidate before product evidence reuse | Require an exact job URL, successful matching job identity, run/SHA, and selected check-run URL |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/github.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
- Before promoting an Action release, run pinned `@v<major>.<minor>.<patch>` consumers on Linux,
  macOS, and Windows; advance `v<major>` only after all pass.
