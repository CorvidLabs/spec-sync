# SpecSync 6.0.0 overnight brief

Written 2026-09-09 for an autonomous session whose job is to take `main` from "6.0.0-ready on paper"
to "6.0.0 proven by dogfooding", with a measured confidence of at least 95 percent, and leave one
final PR plus a written report for the human who publishes. Every claim below was true when written;
verify the starting state before trusting it.

## Starting state

| Item | State |
|---|---|
| `main` | `bc344615` after PR #765 (docs sweep + `docs/RELEASING.md`) and PR #766 (`check --fix` row placement, `--enforcement` help default, SCOPE.md), plus this brief's own PR #767. Any local checkout may be stale: `git fetch origin` and work from `origin/main`, never from an old branch such as `leif/specsync-6-release` (which sits at `ffba9a32`, where the guards still say three and `SDD_VERSION` is 5.0.0). |
| Version | `Cargo.toml` `6.0.0`; `action.yml` default `6.0.0`; `SDD_VERSION` `6.0.0`. |
| Last qualified RC | `v6.0.0-rc.16` at `ffba9a32` (run 34172484287). It does **not** cover the current tree. No `v6.0.0` tag exists. |
| Published channels | GitHub pre-release assets: rc.14. crates.io: 5.2.0. Homebrew tap: 5.2.0. |
| Consumers (#647) | All four known consumers pass an explicit `version: 6.0.0-rc.12`: corvid-account, podo-web and raven via `uses: CorvidLabs/spec-sync@29392630 # v6.0.0-rc.12`, podo-android via `@v6.0.0-rc.12`. None floats `latest`, so repointing `releases/latest` is safe. (Issue #647's table and `docs/RELEASING.md` section 1 describe the pre-migration state; correct the runbook line.) |
| Open PRs | None expected. If one exists, read it; do not replay `review`/`ship` on a package that is already archived (`change review` requires `verifying`); verify its checks and leave the merge to the human. |
| Remote branches | `main`, `leif/mcp-security-history-recovery` (56 unmerged July commits; leave it). |
| Issues | Already closed with citations on 2026-09-09: #677 #672 #603 #667 #660 #648 #631 #641 #647 #615. Close at the stable tag, not before: #628 (`action.yml` default `6.0.0` is valid once `v6.0.0` exists). New: #768 (three `check --fix` scanner edge cases deferred from the #766 review). The other 47 open issues were triaged as defer-to-6.x; re-triage only what your drills touch, and cite a commit when you close anything. |
| Known 6.0.x defects (not blockers) | #656 deleting `verification-attempts.json` bypasses the empty-attempts guard at finalize; #653 malformed JSON config passes silently in `rules`/`rehash`/`compact`; #416 importer writes `files: []` skeletons (docs caveat landed); #615 second half (placeholder rows counted as documented); #675 version skew reads as "invalid change ID"; #690 `ship-status` suggests `check --commit` from `Accepted` (legacy path only); #768 three `check --fix` table-scanner edge cases deferred from the #766 review (two tables in one subsection, indented literal fence marker, fenced `###` line before the table), each needing a drill and a regression test. |
| Release runbook | `docs/RELEASING.md` (verified against `release.yml`, `rc-assets.yml`, validators). |
| CI confidence model | `docs/ci-confidence.md`. Ubuntu + macOS are the required platforms; Windows is neither built nor qualified. |

## Definition of done

1. A written, evidence-backed confidence report at `docs/6-0-confidence-report.md` scoring a fixed
   checklist of verifiable claims (see "Confidence method"). Score ≥ 95 percent, zero open P1.
2. Every defect found that a first-time 6.0 user would hit is fixed with tests on `main`, through the
   SDD lifecycle, or is explicitly listed as deferred with the reason.
3. One final PR (more only if a fix needs its own spec ownership) with all change packages archived
   on the PR, all Codex/corvid-agent threads addressed, all required checks green, **left open for
   the human to inspect and merge in the morning**. Do not merge it yourself; do not close issues
   fixed tonight until it merges (comment "fixed in PR #N" instead).
4. A handoff section at the end of the report listing the exact human steps remaining: cut the RC,
   dispatch `promote`, `cargo publish`, Homebrew bump, changelog date, `v6` floating tag.

## Hard rules

- **Never** create `v6.0.0` or any `-rc.N` tag, dispatch `promote`, run `cargo publish`, touch the
  Homebrew tap, force-push `main`, delete tags, or change security-alert state. Those are human
  actions. A `workflow_dispatch` of `release.yml` with `dry_run=true` is allowed.
- Every meaningful change (`src/`, `tests/`, `site/`, `.github/`, `action.yml`, `Cargo.*`,
  `.specsync/{sdd.json,config.toml,config.json,version}`, and any manifest or lockfile; the
  authoritative list is `meaningful_paths` in `.specsync/sdd.json`) goes through one SDD change
  package:
  `change new` → `answer` → artifacts → `approve --actor 0xLeif` → implement → `check --commit` →
  push → CI green → `review --reviewer 0xLeif` → `ship` (no commit between review and ship) →
  commit the archive tip → push → (human merge). Approval and review identity: the owner decided,
  in the session that wrote this brief, to pre-authorize definition approvals and scoped reviews
  recorded as `0xLeif` for work inside this brief's scope, so the run is not blocked overnight.
  That is a disclosed delegation, not a human inspection of each generated scope, so every
  `change approve` must carry `--note "owner pre-authorization per docs/6-0-overnight-brief.md;
  human inspection at PR merge"` (`change review` has no note flag; its provenance is the PR
  body), the final PR body must list every package approved and reviewed this way, and the
  morning human inspects the PR before merging. A change outside this brief's scope stops and
  waits for the owner.
- Production source needs a declared canonical owner: on `change new`, pass `--spec <module>` for
  every module whose spec `files:` lists a touched `src/` path, or finalize/`ship` refuses with
  "production source without deterministic canonical ownership". When the spec text does not
  change, declare it with `change new ... --spec <module> --no-spec-change --rationale "<why>"`
  (the rationale is mandatory); otherwise write `deltas/<module>.md` with `## ADDED` /
  `## MODIFIED` requirement blocks and let `change check` materialize them.
- Finalize one change at a time. Never merge with an active `.specsync/changes/` entry. After
  another PR merges, rebase, then `check --commit` again, then review, then ship.
- Squash merge only. GitHub refuses merge while any review thread is unresolved; resolve threads
  you have addressed via GraphQL `resolveReviewThread`. corvid-agent may leave changes-requested
  on an old tip and take hours to re-review; address its points, comment, re-request, and continue.
  Merging over a stale block needs `gh pr merge --squash --admin`, which the owner allows for work
  in this brief.
- `fledge lanes run pre-push` before every push; `fledge lanes run verify` before calling a change
  done. Do not put `cargo test` into the Trust lane.
- Do not rewrite historical evidence under `.specsync/archive/`. Do not hand-edit
  `.specsync/change-sequence.json`. Ignore the untracked `.agents/` directory.

## Traps already paid for

- In zsh, never name a shell variable `path` (it is `$PATH`); scripts silently lose every command.
- `touch` the edited source before `cargo build` in a worktree; cargo may report `Finished` on a
  stale binary.
- Never swallow command output (`>/dev/null 2>&1`) in a probe; three "product defects" were
  fixture mistakes.
- `Accepted` is not a resting state in the 6.0 lifecycle (`finalize` accepts and archives in one
  transaction); it is visible only after an interrupted finalize (re-run `finalize`) or in
  legacy/reopened records, so defects that need a resting `Accepted` state are legacy-only.
- `change ship` runs the strict suite and can take 10+ minutes; run it in the background.
- The `assert_rows_contiguous` helper in `tests/integration/fix.rs` (from #766) takes the first
  line beginning with `|` anywhere in the spec text as the table start and requires every line
  up to the added row to begin with `|`; a fenced sample with pipe-led lines placed before the
  table trips it. Assert adjacency directly in that case.
- A `git rebase --onto` after the first PR squash-merges carries conflict resolutions forward.
- `gh api graphql` with unquoted `$ids` in zsh does not word-split; use `while read`.

## Confidence method

Confidence is a fraction of independently verified claims, not a feeling. Build the checklist
first, then verify each claim with two independent agents plus one refuter; a claim passes only
when both verifiers pass with concrete evidence (command, output, SHA) and the refuter fails to
refute. Weight P1 claims 3, P2 claims 1. Report the weighted pass fraction. Minimum checklist:

- Fresh project, each of Rust, TypeScript, Python, Swift, Go: `init` → `check` (0 warnings on a
  clean tree; warning on an undocumented export; `--strict` exit 1) → `change adopt` → full
  lifecycle to an archived change on a real PR-shaped branch, in git and (where supported)
  non-git trees, with `--json` outputs parseable at each step.
- Upgrade path: a 5.2.0-initialized project with one in-flight workflow-v1 change follows
  `MIGRATION.md` literally and ends with the v1 change landed and a v2 change archived; a v1 change
  merged after the cutoff is refused with the documented message (#674).
- GitHub Action: `CorvidLabs/spec-sync@<this tree>` on a sandbox repo (`CorvidLabs/spec-sync-sandbox`)
  on `ubuntu-latest` and `macos-latest`; `strict: true` fails on drift; `comment: true` posts.
  Only rc.14 has published release assets, so the download cell must use `version: 6.0.0-rc.14`;
  to exercise the tree's own binary, use the `download-base-url` input against a runner-local
  mirror the way `ci.yml`'s `action-consumer` job does. Never publish assets to make this pass.
- Docs walkthroughs executed literally by an agent that may only follow the text: README quick
  start, `site/src/content/docs/quickstart.md`, `site/src/content/docs/workflow.md`, `MIGRATION.md`,
  `examples/*/README.md`, `docs/RELEASING.md` sections 1 to 3 (dry run only).
- Contract stability: `--help` for every verb matches the CLI reference; the JSON output of
  `check`, `change list`, `change status`, `coverage` and `score` matches its documented shape on
  a clean tree and on a degraded tree (`change list --json` and `change status --json` are
  documented to return a bare array when healthy and an object with `changes`/`unreadable` plus
  a non-zero exit when degraded; that variant is the contract, not a failure); exit codes match
  the documented enforcement modes; unknown config keys warn, unknown
  `state.json` fields survive a read, `workflow_version: 3` is refused with the documented message.
- Tamper drills: edit `approvals.json` actor, delete `verification-attempts.json`, `git mv` an
  archive dir, strip `workflow_version`; each either is refused as documented or is listed as a
  documented boundary (actor labels do not authenticate identity).
- Release lane: `workflow_dispatch` `dry_run=true` against `v6.0.0-rc.16` passes `resolve` and
  `validate`; the 52 release-candidate validator tests pass; both Bash guards require exactly two
  evidence records; consumer pins verified live.
- Performance sanity: `check` on this repository under 2 s; `change list` with 20 draft changes
  measured and reported (do not fix unless trivial; #439).
- MCP: `initialize`, `tools/list` (5 read-only tools; 7 with `--allow-write`), one `specsync_check`
  call, on a fresh project.
- Examples: `examples/quickstart`, `sdd-lifecycle`, `sdd-concurrent-changes`, `sdd-five-epics`
  pass with the release binary built from the tree.

## Fix policy

Fix when a first-time 6.0 user hits it, the fix is local, and a test pins it. Prefer refusing with
a clear message over silently widening behaviour. Candidates in rough priority: #656, #653, #615
second half, #675 message, #690 message. Do not start #439/#645 performance work, #434 unknown-field
preservation, #532 multi-clone approvals, or any feature request. Record everything deferred in the
report with the issue number.

## Where things are

- Lifecycle verbs and precedent: `AGENTS.md`; archived packages under
  `.specsync/archive/changes/2026-09-0*` show the artifact style: always `change.md`,
  `context.md`, `design.md`, `tasks.md`, `testing.md` plus the evidence ledgers; `docs.md`,
  `requirements.md`, `plan.md`, `research.md` and `deltas/<module>.md` appear only when the
  interview selected them or a spec changed.
- Release: `.github/workflows/release.yml`, `docs/RELEASING.md`, `docs/ci-confidence.md`.
- Prior review evidence: `docs/6-0-release-review.md`, `docs/6-0-findings.md`,
  `docs/SESSION-SUMMARY-6-0.md`.
- Drills from earlier sessions: `CorvidLabs/spec-sync-sandbox` (private) `drills/` on its `main`
  holds scripts numbered 001 to 070 with runner lanes; its `drills` workflow was red on `main` at
  its last runs (2026-08-29) and PR #96 there is open. Sandbox PR #28 was closed unmerged; do not
  rely on it or on any earlier pass count. Establish which drills pass against this tree before
  counting them as evidence, and cite the run id.
