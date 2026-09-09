# 6.0.0 overnight proving journal

Append-only. One entry per phase: what ran, what it found, what was decided.

## Phase 0 — orient (started 2026-09-09)

- Worktree `/private/tmp/specsync-6-proving`, branch `leif/specsync-6-proving`, from `origin/main` at the commit shown by `git log -1` in this worktree.
- Release binary built from this tree (see `target/release/specsync --version`).
- Starting-state verification and checklist enumeration recorded below when complete.

### Phase 0 result — starting-state verification (11 rows)

- **OK ** Version: `Cargo.toml` `6.0.0`; `action.yml` default `6.0.0`; `SDD_VERSION` `6.0.0`.
- **OK ** Published channels | GitHub pre-release assets: rc.14. crates.io: 5.2.0. Homebrew tap: 5.2.0.
- **OK ** Open PRs (gh pr list) and remote branches (git ls-remote --heads origin)
- **OK ** Release runbook and ci-confidence rows: docs/RELEASING.md exists and its consumer loop includes raven; REQUIRE
- **OK ** Sandbox: CorvidLabs/spec-sync-sandbox exists, its drills/ directory listing on main, latest drills workflow ru
- **OK ** Hard rules row: meaningful_paths in .specsync/sdd.json includes site/, .github/, action.yml, Cargo.toml; specs
- **DRIFT** Starting state: main tip SHA and merged PRs (#762 #764 #765 #766 #767); local checkout at /private/tmp/specsyn
  - Brief states "`main` | `bc344615` after PR #765 ... and PR #766 ..., plus this brief's own PR #767". Live `origin/main` (and GitHub branch API) is `0d0251bf`, the squash-merge of PR #767 (merged 2026-09-09T06:53:03Z). `bc344615` is the merge commit of PR #766 and is exactly one commit behind the live tip. The brief's SHA was recorded before #767 merged, so the statement is internally inconsistent ("plus PR #767" cannot be true at bc344615). Correct wording: "`main` | `0d0251bf` after PR #765, PR #766, and this brief's own PR #767." No other drift in this aspect: local checkout /private/tmp/spe
- **DRIFT** Issues (Starting state table): "Already closed with citations on 2026-09-09: #677 #672 #603 #667 #660 #648 #63
  - Ten "already closed" issues: match. #628 open: match. #768 exists and open: match. Open-issue count: drift. The brief's "The other 47 open issues" (read as open issues besides #628 and #768) implies 49 open in total; live total is 48, and there are 46 open issues other than #628 and #768. No issue was closed after the brief merged (0 closures after 2026-09-09T06:53:03Z), so this is a miscount or ambiguous wording in the brief, not a state change. The sentence holds only if #628 is counted among the "other 47" (47 = all open issues except #768). Suggested correction: "The other 46 open issues w
- **OK ** Last qualified RC: `v6.0.0-rc.16` at `ffba9a32` (run 34172484287). It does not cover the current tree. No `v6.
- **OK ** Consumers (#647): corvid-account, podo-web, podo-android, raven workflow files pin version 6.0.0-rc.12
- **OK ** Known 6.0.x defects (not blockers): #656 deleting `verification-attempts.json` bypasses the empty-attempts gua

Decision: both drifts are wording only (SHA recorded before #767 merged; issue count off by two). Corrected in the brief on this branch. No state change.

## Phase 1 — dogfood matrix (22 cells) and Phase 2 — adversarial drills (11 cells)

Launched in parallel against `target/release/specsync` built from `0d0251bf`. Sandbox `drills.yml` dispatched against the same SHA (run 34350039719 on the self-hosted Mac runner). Results appended when complete.

### Fan-out (Opus)

Five parallel fronts against the same binary (`0d0251bf`):

| Front | Scope |
|---|---|
| Phase 1 dogfood matrix | 22 cells: five languages, five tree kinds, full lifecycle, 5.2.0 upgrade both directions, four docs walkthroughs, examples, MCP, perf, cargo package, GitHub Action on the sandbox |
| Phase 2 adversarial drills | 11 cells: help-vs-docs, JSON shapes, exit codes, tolerance probes, four tamper drills, concurrency, `check --fix`, `import` |
| Phase 2b verb coverage | 8 cells: scaffold/generate/new, coverage/report/score/deps, hooks+agents install, watch/merge/compact, diff/comment/resolve, schema-SQL, MCP security, config precedence |
| Phase 2c docs + source hunt | 9 finders (5 doc pages, 4 source audits: release lane, digests, check on hostile input, concurrency/locks) then 3 refuters per candidate finding |
| Sandbox drills | `drills.yml` dispatched against this exact SHA on the self-hosted Mac runner (run 34350039719) |

Plus `fledge lanes run verify` locally for claim C41.

### Phase 1 result — dogfood matrix (23 cells)

| Cell | Claims | Status | Findings |
|---|---|---|---|
| `action-sandbox` | C17, C18, C19 | pass | 4 |
| `cargo-package` | C42 | fail | 2 |
| `crlf` | C10 | fail | 3 |
| `examples` | C24 | pass | 1 |
| `fresh-go` | C05 | pass | 4 |
| `fresh-python` | C03, C13 | pass | 4 |
| `fresh-rust` | C01, C13, C16 | pass | 7 |
| `fresh-swift` | C04 | pass | 2 |
| `fresh-ts` | C02, C13 | pass | 6 |
| `lifecycle-python` | C08 | pass | 4 |
| `lifecycle-rust` | C06, C13 | pass | 5 |
| `lifecycle-ts` | C07 | pass | 2 |
| `mcp` | C40: MCP `initialize`, `tools/list` (5 t | fail | 3 |
| `monorepo-subdir` | C11 | pass | 2 |
| `non-git` | C12 | fail | 3 |
| `perf` | C39 | pass | 1 |
| `readme-quickstart` | C20 (P1): README quick start executed li | fail | 2 |
| `releasing-dry-run` | C25, C37 | pass | 4 |
| `shallow-clone` | C09 | fail | 3 |
| `site-quickstart` | C21 | fail | 7 |
| `site-workflow` | C22: site/src/content/docs/workflow.md e | pass | 4 |
| `upgrade-52` | C14, C23 | fail | 7 |
| `upgrade-52-late-v1` | C15 | fail | 2 |

### Phase 2 result — adversarial drills (11 cells)

| Drill | Status | Findings |
|---|---|---|
| `check-fix` | pass | 5 |
| `concurrency` | fail | 7 |
| `exit-codes` | pass | 7 |
| `help-vs-cli-doc` | fail | 7 |
| `import` | pass | 4 |
| `json-shapes` | pass | 4 |
| `tamper-approvals` | pass | 1 |
| `tamper-archive-mv` | pass | 3 |
| `tamper-attempts` | fail | 3 |
| `tamper-workflow-version` | pass | 2 |
| `tolerance` | pass | 3 |

**Totals:** 128 findings across 34 cells — P1 4, P2 26, P3 98.

P1 findings:

- **documentation-error** (`readme-quickstart`) — README Quick start `specsync check --strict` exits 1 immediately after `specsync add-spec auth`
- **defect** (`cargo-package`) — cargo publish --dry-run fails: src/change.rs include_str!s .github/scripts/lifecycle-validation-limits.json which is not in the crate include set
- **defect** (`crlf`) — ship/finalize fails post-move validation when the change delta file is CRLF in the working tree (core.autocrlf=true checkout)
- **defect** (`tamper-attempts`) — Deleting verification-attempts.json bypasses the empty-attempts guard; ship finalizes and archives a regenerated 1-entry ledger (#656)

Fix waves launched (isolated worktrees, tests first): wave 1 = attempts-ledger tamper (#656), fence-blind symbol reader + #768.3, emitted `check --spec` command, cli.md drift. Wave 2 = cargo publish include set, CRLF finalize, first-run docs, `generate` no-op.


### Baseline verify (claim C41, pristine tree)

`fledge lanes run verify` on `0d0251bf`: **pass in 19m 55s** (fmt, clippy, check-types, full test suite, release build, strict specs, 52 release-validator tests). Must be re-run on the final fix tree before the confidence report is written.

### Loop-until-dry, round 2

Launched a second finder round plus a completeness critic over surfaces no earlier cell touched: Action inputs beyond version/strict/comment, the VS Code extension, the whole `lifecycle` verb family, `view`/`issues`/`rules`/`rehash`/`stale`/`changelog`, ignore-path handling and 200-file scale, and a systematic refusal-message audit (25+ refusals judged on what/why/next-action). The critic audits coverage itself: which checklist claims have no evidence, which passes rest on thin evidence, which findings assert an unproven cause.

### Phase 2b result — verb coverage (8 cells, 68 findings: 5 P1, 26 P2, 37 P3)

| Drill | Status | Findings |
|---|---|---|
| `config-precedence` | fail | 5 |
| `coverage-report-score` | fail | 13 |
| `diff-comment-resolve` | fail | 15 |
| `hooks-agents-install` | pass | 6 |
| `mcp-security` | fail | 6 |
| `scaffold-generate-new` | fail | 12 |
| `schema-sql` | fail | 4 |
| `watch-merge-compact` | fail | 7 |

P1 findings:

- **defect** (`scaffold-generate-new`) — `specsync new` mints a spec that can never pass `specsync check` — it omits 3 of the 7 required_sections `specsync init` configured
- **defect** (`coverage-report-score`) — report --require-coverage gate is skipped whenever any module is stale, incomplete, or has unmeasurable staleness (#605, broader than documented)
- **defect** (`config-precedence`) — A parse-failed config.toml or config.json is silently replaced by built-in defaults on rules/rehash/compact/deps/archive-tasks/view, which exit 0 and write files (#653 confirmed, #583 fix incomplete)
- **defect** (`mcp-security`) — Read-only MCP server spawns PATH-resolved `git` with the full unsanitized environment, exposing GITHUB_TOKEN and other secrets
- **defect** (`watch-merge-compact`) — compact, archive-tasks, watch and merge escape the #570 unloadable-config choke point — compact and archive-tasks print a green all-clear and exit 0 over a project they never examined

Fix wave 3 launched: `new` required sections + name rules, config fail-open choke point (#653/#583), MCP child-process environment sanitization, scaffold path containment.


### Fix waves in flight — where to resume

Twelve fixes are being written in isolated git worktrees under
`/Users/leif/Development/_CorvidLabs/spec-sync/.claude/worktrees/`, each branched from
`origin/main` at `0d0251bf`, each left uncommitted for the orchestrator to collect:

| Run | Fixes |
|---|---|
| `wf_199b7ff5-f5c-{1..4}` | attempts-ledger tamper (#656); fence-blind symbol reader + #768.3; emitted `check --spec` command that does not parse; cli.md drift |
| `wf_b264e6fd-465-{1..4}` | `cargo publish` include set (P1 blocker); CRLF finalize (P1); README + site quickstart first-run (P1); `generate` silent no-op |
| `wf_65e75083-0c1-{1..4}` | `new` required sections + name rules (P1); config fail-open choke point (#653/#583, P1); MCP child-environment sanitization (P1 security); scaffold path containment |

To resume: for each worktree, `git -C <path> diff` and apply onto a branch off current `main`,
then build one change package covering every touched path (`--spec change --spec cmd_check
--spec cmd_init --spec cli_args ...` per the specs that own those files), approve, `check --commit`,
review, ship, and open the fix PR. Re-run `fledge lanes run verify` on the assembled tree before
scoring the checklist; the baseline pass recorded above was on the pristine tree.

Do not merge the fix PR unattended: the brief requires the human to inspect and merge.
