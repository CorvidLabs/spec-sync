# SpecSync 6.0.0 confidence report

Written 2026-09-09 against `origin/main` at `7fc6912` (squash of [#770](https://github.com/CorvidLabs/spec-sync/pull/770); parent `0fc2b40` is squash of [#769](https://github.com/CorvidLabs/spec-sync/pull/769)) plus the remaining fix PR [#771](https://github.com/CorvidLabs/spec-sync/pull/771) (`leif/specsync-6-p1-review-followup`). Product tip `9f64bef`. Archive tip `7c523a4`. **#769 and #770 are merged. Do not merge #771 unattended.**

Checklist: `docs/6-0-confidence-checklist.md` on #769 (C01–C42, P1=3, P2=1). A claim passes when overnight proving (verifier 1, journal on #769) and this session (verifier 2) agree, or when overnight failed on a P1 this tree fixed and this session re-ran it. Overnight cells that failed and were **not** re-drilled stay fail.

## Score

| | |
|---|---|
| Weighted pass | **83 / 90 = 92.2%** |
| Open first-user P1 defects | **0** (eight overnight P1s closed on #770; two first-user P1s #770 introduced closed on #771) |
| Target | ≥ 95%, zero open P1 |
| Gap to 95% | 3 weighted points (one more P1). Closing C14 **or** C23 by re-running the 5.2.0 upgrade walkthrough would land 86/90 = 95.6%. |

The 95% bar is missed because the 5.x → 6.0 upgrade walkthrough (C14, C23) was not re-executed on the fix tree. Those are existing-consumer claims, not first-run P1s. Every defect a first-time 6.0 user hits on `init` / `check` / `new` / `cargo publish` / MCP / CRLF / README is fixed and tested on `main` (#770) or on #771.

## What shipped on #770 (now on `main`)

Package `close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving`, archived at `.specsync/archive/changes/2026-09-09-close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving/`. Squash-merged 2026-09-09T14:31:23Z as `7fc6912`.

Approve and review actor: `0xLeif`, with `--note "owner pre-authorization per docs/6-0-overnight-brief.md; human inspection at PR merge"` (disclosed overnight-brief delegation).

| P1 (overnight) | Fix | Evidence |
|---|---|---|
| `cargo publish` `include_str!` of `.github/scripts/lifecycle-validation-limits.json` | Bundled `src/lifecycle-validation-limits.json` (byte-identical) | `cargo publish --dry-run --locked` compiled and aborted the upload; `cargo package --list` contains the file. C42. |
| `specsync new` omitted 3 of 7 `init` required sections | Reuses `generate_spec`; scaffold name rules | `new_auto_detects_single_source_file`, `new_refuses_reserved_and_invalid_module_names` |
| Malformed JSON config silently defaulted (#653 / remaining #583) | `load_error` on JSON parse-fail; `load_config` fail-closed | `malformed_json_config_refuses_rules_rehash_compact_and_view` |
| MCP `git` inherited `GITHUB_TOKEN` | `git_cmd`: `env_clear` + allowlist | `git_inherited_env_excludes_secrets_and_git_overrides` |
| `ship` refused a CRLF checkout | `canonical_definition_artifact_payload` folds `\r\n` → `\n` | `canonical_definition_payload_folds_crlf_to_lf` |
| README quick start failed at step 1 | Source file before `add-spec`; no `--strict` on a stub | Executed literally: `init` → write `src/auth.ts` → `add-spec` → `check` exit 0. `readme_quick_start_init_add_spec_and_check_succeed` |
| Deleting `verification-attempts.json` let `ship` finalize (#656) | Missing file == empty ledger; Verifying refuses to recreate | `deleting_verification_attempts_refuses_adopted_finalization`, `emptying_verification_attempts_refuses_adopted_finalization`, `verifying_change_refuses_to_recreate_a_missing_attempts_ledger` |
| Public API reader counted fenced backticks; `--fix` saw fenced `###` (#768.3) | `blank_fenced_code` preserves byte length | `fenced_example_backticks_are_not_documented_exports`, `fix_ignores_fenced_exported_heading_before_the_table` |

Also in the same package: `check --spec NAME` is a real repeatable clap flag; `generate` fails closed on empty writes (`skipped_no_files`); `scaffold --dir` is confined.

## What shipped on #771 (leave open)

Two packages, both archived on the PR. Approve and review actor: `0xLeif`, same overnight-brief note. Human squash-merges.

### Package 1 — `stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository`

Codex review of #770 found two first-user P1s the overnight package introduced.

| P1 (#770 regression) | Fix | Evidence |
|---|---|---|
| `check --fix` space-wiped fenced Public API examples when renaming a near-miss heading | `fix_near_miss_headers` scans a fence-blanked copy and writes the original section | `fix_preserves_fenced_example_when_renaming_a_near_miss_header` |
| `GIT_CEILING_DIRECTORIES=parent(root)` made `git -C repo/sub` fail | Default `git_cmd` does not pin a ceiling; nested `is_git_repo` is true | `is_git_repo_detects_project_inside_repository_subdirectory` |

`cmd_new` spec body synchronized to `generate_spec` / `validate_scaffold_module_name` / `check_case_collision`.

### Package 2 — `isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up`

Codex P1 on #771: dropping the global ceiling re-broke REQ-mcp-008 when `TMPDIR` points inside a host worktree. The snapshot has no `.git`, walk-up finds the host, `score_spec` treats freshness as measurable, and `score_spec_for_mcp` deducts again.

| P1 (#771 Codex) | Fix | Evidence |
|---|---|---|
| MCP snapshot inside a host worktree discovers host git | Default `git_cmd` stays ceiling-free; MCP snapshot dispatch wraps git probes with `with_discovery_ceiling(parent(snapshot))` | `discovery_ceiling_isolates_a_subdirectory_from_host_walk_up`, `discovery_ceiling_does_not_leak_after_the_closure`, `mcp_snapshot_inside_host_worktree_does_not_discover_host_git_or_double_penalize` |

`specs/cmd_new` companions no longer name `chrono_lite_today` / `get_exported_symbols` as the living contract.

CI on product HEAD `9f64bef` (run [34368108936](https://github.com/CorvidLabs/spec-sync/actions/runs/34368108936); trust [34368108916](https://github.com/CorvidLabs/spec-sync/actions/runs/34368108916)): test, coverage, spec-check, trust, Required CI gate, SpecSync implementation ready, SpecSync scoped review — all SUCCESS.

Local verify-lane equivalent on the fix tree (fledge CLI not installed in this sandbox; steps from `fledge.toml` `[lanes.verify]` / AGENTS.md pre-push):

- `cargo fmt --check` pass
- `cargo clippy -- -D warnings` pass
- `cargo test` (CI authority): pass on GitHub. Local euid=0 makes two `0o000`-permission unit tests fail; CI is the runner that matters.
- Targeted pins: `discovery_ceiling_*`, `mcp_snapshot_inside_host_worktree_does_not_discover_host_git_or_double_penalize`, `test_mcp_score_always_marks_git_freshness_unavailable_and_fails_closed`
- `specsync check --strict --require-coverage 100 --force`: 62/62, 106/106 files, 147784/147784 LOC
- `python3 .github/scripts/test-validate-release-candidate.py`: 52 tests OK (from the #770 session; not re-run on this tip)

## Claim table

V1 = overnight journal on #769 against `0d0251bf`. V2 = this session against `7fc6912` + #771 product `9f64bef` / archive `7c523a4`.

| # | W | V1 | V2 | Result | Evidence |
|---|---|---|---|---|---|
| C01 | P1 | pass | pass | **pass** | journal `fresh-rust`; CI + lib tests |
| C02 | P1 | pass | pass | **pass** | journal `fresh-ts`; README quick start used TS |
| C03 | P1 | pass | pass | **pass** | journal `fresh-python` |
| C04 | P2 | pass | — | **pass** | journal `fresh-swift`; not re-run |
| C05 | P2 | pass | — | **pass** | journal `fresh-go`; not re-run |
| C06 | P1 | pass | pass | **pass** | journal `lifecycle-rust`; two packages archived on a real PR (#771) after #770 landed |
| C07 | P1 | pass | — | **pass** | journal `lifecycle-ts` |
| C08 | P2 | pass | — | **pass** | journal `lifecycle-python` |
| C09 | P2 | fail | pass | **pass** | Product names `incomplete shallow Git checkout` and tells the operator to fetch through the recorded base (`src/change.rs`; tests `shallow_history_with_corrections_fails_closed`). Remedy is fetch-through-base, not the literal `fetch-depth: 0` string. |
| C10 | P2 | fail | pass | **pass** | CRLF fold in `canonical_definition_artifact_payload`. Tests `canonical_definition_payload_folds_crlf_to_lf`, `canonical_tasks_payload_folds_crlf_before_checkbox_rewrite`. |
| C11 | P2 | pass | pass | **pass** | journal `monorepo-subdir`; `is_git_repo_detects_project_inside_repository_subdirectory` (ceiling-free default) |
| C12 | P2 | fail | pass | **pass** | `init_then_check_is_usable_without_git_and_does_not_nag_about_legacy_layout`; `stale_outside_git_repo_fails_with_message`. Overnight cell had extra P3s; the claim (init/check/coverage/score work; lifecycle refuses, no panic) holds. |
| C13 | P1 | pass | — | **pass** | journal JSON cells |
| C14 | P1 | fail | not re-run | **fail** | 5.2.0 in-flight v1 → MIGRATION.md → archived v2. Overnight `upgrade-52` failed (7 findings). Not reconstructed here. **This is the 95% gap.** |
| C15 | P1 | fail | pass | **pass** | #674 already closed; `first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff` and `workflow_v2_cannot_downgrade_by_omitting_workflow_version` pass. Overnight `upgrade-52-late-v1` cell failed on message wording; the refusal exists. |
| C16 | P1 | pass | pass | **pass** | journal `exit-codes`; `check_validation_errors_exit_nonzero_by_default`, `warn_mode_exits_0_even_with_errors` |
| C17 | P1 | pass | pass | **pass** | journal `action-sandbox`; CI job `Packaged GitHub Action consumer` SUCCESS on #771 |
| C18 | P2 | pass | — | **pass** | journal macos action cell |
| C19 | P2 | pass | — | **pass** | journal `comment: true` |
| C20 | P1 | fail | pass | **pass** | README executed: `init` → `src/auth.ts` → `add-spec auth` → `check` exit 0 (13 warnings, 0 failed). `readme_quick_start_init_add_spec_and_check_succeed`. |
| C21 | P1 | fail | pass | **pass** | site quickstart now matches README (source file first, no `--strict` on a stub). Same execution as C20. |
| C22 | P1 | pass | pass | **pass** | journal `site-workflow`; #770 and both #771 packages ran that lifecycle on a real PR |
| C23 | P1 | fail | not re-run | **fail** | Literal `MIGRATION.md` on a 5.2.0 tree. Same overnight `upgrade-52` cell as C14. |
| C24 | P2 | pass | — | **pass** | journal `examples`; `examples/quickstart` integration tests pass |
| C25 | P2 | pass | — | **pass** | journal `releasing-dry-run` against `v6.0.0-rc.16`. This session did **not** dispatch `release.yml`. |
| C26 | P1 | fail | pass | **pass** | Overnight `help-vs-cli-doc` was `--spec` drift. `cli.md` now documents repeatable `check --spec NAME`; clap flag exists (`check_accepts_repeatable_spec_flag`). Residual P3 help mismatches, if any, are not first-run blockers. |
| C27 | P1 | pass | — | **pass** | journal `json-shapes` |
| C28 | P2 | pass | — | **pass** | journal `exit-codes` |
| C29 | P2 | pass | — | **pass** | journal `tolerance` |
| C30 | P1 | pass | — | **pass** | journal `tamper-approvals` |
| C31 | P1 | fail | pass | **pass** | #656 closed in product. Three unit tests above. Issue left OPEN with "fixed in PR #770". |
| C32 | P1 | pass | — | **pass** | journal `tamper-archive-mv` |
| C33 | P1 | pass | — | **pass** | journal `tamper-workflow-version`; `workflow_v2_cannot_downgrade_by_omitting_workflow_version` |
| C34 | P2 | fail | not fixed | **fail** | Overnight `concurrency` cell. Brief: do not start #532 multi-clone approvals. Deferred. |
| C35 | P1 | pass | pass | **pass** | journal `check-fix` plus fence-aware `--fix` tests, including `fix_preserves_fenced_example_when_renaming_a_near_miss_header` |
| C36 | P2 | pass | — | **pass** | journal `import`; #416 caveat already on main |
| C37 | P1 | pass | pass | **pass** | journal dry-run + 52 validator tests OK in the #770 session. No `promote`, no tag. |
| C38 | P2 | pass | — | **pass** | journal consumer pins; `docs/RELEASING.md` still lists all four at `6.0.0-rc.12` |
| C39 | P2 | pass | pass | **pass** | `check` on this repo 0.991 s cached, 5.5 s `--force` (measured on the #770 tree). Claim is under 2 s for `check`; cached path meets it. `change list` with 20 drafts not re-measured (#439). |
| C40 | P1 | fail | pass | **pass** | `mcp initialize` → protocolVersion 2024-11-05, server `specsync` 6.0.0. `tools/list` returns 5 read-only tools. Git children no longer inherit `GITHUB_TOKEN`. MCP snapshots pin `GIT_CEILING_DIRECTORIES` to the snapshot parent so a tempfile inside a host worktree cannot walk up. |
| C41 | P1 | pass (pristine) | pass (fix tree) | **pass** | Baseline `fledge lanes run verify` 19m55s on `0d0251bf`. Fix tree: clippy + CI test + spec-check 62/62 100% + trust SUCCESS. |
| C42 | P2 | fail | pass | **pass** | `src/lifecycle-validation-limits.json` in the crate; `cargo publish --dry-run --locked` compiled (upload aborted). |

**Pass weight:** 22 × 3 + 17 × 1 = 83. **Fail:** C14 (3), C23 (3), C34 (1).

## Deferred (with issue numbers)

| Item | Why deferred |
|---|---|
| C14 / C23 5.2.0 upgrade walkthrough | Overnight cell failed; needs a real 5.2.0 fixture and a literal MIGRATION.md run. Not a first-run P1. Re-run before calling 95%. |
| C34 two-clone slug collision (#532) | Brief: do not start #532. |
| #605 `report --require-coverage` skipped when any module is stale | Not a clean-tree first run. |
| #615 second half (placeholder rows count as documented) | Out of #766's scope; still open. |
| #768.1 two tables in one subsection; #768.2 indented fence marker | Only #768.3 is in this tree. Fenced `## Public API` locator is 6.0.1. |
| #675 / #690 message polish | Brief: fix-policy candidates, not first-run blockers. |
| #439 / #645 performance | Brief: do not start. `check` is already under 2 s cached. |
| #434 unknown-field preservation | Brief: do not start. |
| #416 `import` writes `files: []` skeletons | Docs caveat already on main; C36 pass. |
| Compact/watch/merge JSON choke | Now goes through `load_config()`; covered by #653 fix. Residual TOML-only holes, if any, are not the P1. |
| Generator `Created \| <date>` changelog row | Codex P2 on #770; same as `add-spec` / `generate`. |
| Extra `config.required_sections` headings beyond the seven `init` writes | Codex P2 on #770. |
| Pre-upgrade CRLF approval digest compare | Existing-consumer C14/C23. |
| `include_str!` of `.github/scripts/lifecycle-validation-limits.json` from `change_tests` | Packaged-crate `cargo test` on crates.io; 6.0.1. |
| `scaffold --dir` symlink-follow | Codex on merged #770; 6.0.1. |

## Hard rules honored

No tags. No `promote`. No `cargo publish` (dry-run only, on #770). No Homebrew. No force-push of `main`. No issue close. #771 left open. Codex threads on #770 and #771 addressed and resolved, not dismissed.

## Human remaining steps (`docs/RELEASING.md`)

Inspect and **squash-merge [#771](https://github.com/CorvidLabs/spec-sync/pull/771)** (both change packages archived; product CI green on `9f64bef`; wait for archive-tip CI after this push). #770 and #769 are already on `origin/main`.

After #771 is on `origin/main`:

1. **Cut the next RC** (do not reuse a name whose earlier run carried a different SHA):

```bash
git fetch origin main
git tag -a v6.0.0-rc.17 <sha-on-origin/main> -m "SpecSync 6.0.0 release candidate 17"
git push origin refs/tags/v6.0.0-rc.17
gh run watch <qualify-run-id>
```

Last qualified RC is still `v6.0.0-rc.16` at `ffba9a32` and does **not** cover this tree.

2. **Dry run** (allowed by the brief; this session did not dispatch it):

```bash
gh workflow run release.yml --ref main -f rc_tag=v6.0.0-rc.17 -f dry_run=true
```

3. **Promote** (human only — `promote` has never executed in this repository):

```bash
gh workflow run release.yml --ref main -f rc_tag=v6.0.0-rc.17
```

4. **crates.io** (currently 5.2.0). From the tagged tree: `cargo publish --dry-run --locked` then `cargo publish --locked`. The include-set hole that blocked this is fixed on `main` via #770.

5. **Homebrew** tap `CorvidLabs/homebrew-tap` `Formula/spec-sync.rb`: version `6.0.0` and four sha256s from the `.sha256` sidecars (not musl).

6. **Changelog date.** `## [6.0.0]` has no date. After the stable tag, PR `## [6.0.0] - YYYY-MM-DD` and drop the "stable publication is pending" line.

7. **Floating `v6` tag** by hand after platform smoke, not by `release.yml`.

Optional before promote: re-run C14/C23 on a 5.2.0 fixture if the 95% bar is a publish gate. Not required to merge #771.

Issues #656, #653, #768 remain OPEN with "fixed in PR #770" comments (do not close until the human decides; #768 only .3 is fixed). #583 CLOSED historically.
