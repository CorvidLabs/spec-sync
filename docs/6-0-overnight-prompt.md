# Overnight prompt for the 6.0.0 proving session

Paste everything below the line into a fresh Claude Code session (Opus, ultracode on) started in
`/Users/leif/Development/_CorvidLabs/spec-sync` on an up-to-date `main`.

---

ultracode

You are running unattended overnight in the CorvidLabs/spec-sync repository. Your mission: take
SpecSync 6.0.0 from "ready on paper" to "proven by dogfooding", with a measured confidence of at
least 95 percent, fix what you find through the repository's own verified-change lifecycle, and
leave one final pull request plus a written report for the human who publishes in the morning.
Nobody will answer questions until then. Make routine decisions yourself, record them, and keep
going until the definition of done is met or you are genuinely blocked on a human-only action.

Read these first, in this order, before doing anything else:
1. `docs/6-0-overnight-brief.md` — starting state, hard rules, traps, the confidence method, the
   fix policy. It is the contract for this session. If the file is missing, stop and say so.
2. `AGENTS.md` — the lifecycle verbs, the four ordering rules, the pre-push rule.
3. `docs/RELEASING.md` and `docs/ci-confidence.md` — the release lane you are proving but must not
   trigger.
4. `docs/6-0-release-review.md`, `docs/6-0-findings.md`, `docs/SESSION-SUMMARY-6-0.md` — what
   earlier sessions found and how they were burned.

Hard rules, restated because they are the ones that matter unattended: never create any tag,
never dispatch `promote`, never `cargo publish`, never touch the Homebrew tap, never force-push
`main`, never change a security alert. `release.yml` with `dry_run=true` is the only release
dispatch you may make. Every meaningful change goes through one SDD change package and is
archived on its PR before merge; the owner (0xLeif) pre-authorizes definition approvals and
scoped reviews recorded as `--actor 0xLeif` / `--reviewer 0xLeif` for work inside the brief's
scope, and allows `gh pr merge --squash --admin` over a stale automated block once every review
thread is addressed and resolved. Run `fledge lanes run pre-push` before every push and
`fledge lanes run verify` before calling any change done.

How to work. Use workflows for every substantive phase; you have the budget for roughly fifty
subagents at a time and should use it. Pipeline by default, barrier only when a phase needs all
prior results. Adversarially verify everything: a finding is real only when an independent
skeptic fails to refute it, and a fix is done only when a fresh agent reproduces the original
defect on the old binary and confirms it gone on the new one. Loop until dry: keep sending finders
until two consecutive rounds surface nothing new. Never cap coverage silently; log what you
skipped. Keep a running journal at `docs/6-0-overnight-journal.md` (append-only, one entry per
phase: what ran, what it found, what you decided) so the morning reader can reconstruct the night
without the transcript.

Phases, each its own workflow, each read and judged by you before the next starts:

Phase 0, orient (10 to 15 agents, read-only). Verify the brief's starting-state table against the
live repository and GitHub; note any drift in the journal. Check whether PR #766 merged; if it is
open and green, finish its lifecycle (review, ship, archive tip, merge) exactly as `AGENTS.md`
describes. Build the release binary from `main` (`cargo build --release`; `touch` the sources
first) and keep its path. Enumerate the confidence checklist from the brief into concrete,
numbered claims with a P1/P2 weight each and write it to `docs/6-0-confidence-checklist.md`
before any drill runs; the checklist is fixed from this point, and later discoveries append,
never edit.

Phase 1, dogfood matrix (30 to 50 agents, each in its own scratch directory under
`/private/tmp/claude-501/`, never inside the repository). One agent per cell: language (Rust,
TypeScript, Python, Swift, Go) × tree kind (git, non-git where supported, shallow clone, CRLF
checkout, monorepo subdirectory) × path (fresh `init` → `check` → `change adopt` → full lifecycle
to an archived change; and the 5.2.0 upgrade path from `MIGRATION.md` with one in-flight
workflow-v1 change). Agents follow the public docs literally and may not read source to make a
step work; when the text and the binary disagree, that is a finding. Every agent records the
exact commands, outputs and exit codes. Include the GitHub Action cells on
`CorvidLabs/spec-sync-sandbox` (ubuntu and macos runners, `strict: true` drift failure,
`comment: true`), the MCP smoke cells, the examples, and the release-lane dry run against
`v6.0.0-rc.16`.

Phase 2, adversarial and contract drills (15 to 25 agents). Tamper drills from the brief; `--help`
of every verb against `site/src/content/docs/cli.md`; JSON shape and exit-code stability across
a clean and a degraded tree; unknown config keys and unknown `state.json` fields;
`workflow_version: 3` refusal; concurrency (two clones, one slug, one shared file); the
`check --fix` and `import` behaviours; performance sanity numbers. Each drill states expected
versus observed and classifies the difference as defect, documented boundary, or documentation
error.

Phase 3, triage and fix. Deduplicate all findings, then have three independent skeptics try to
refute each one; keep only what survives. Rank by "would a first-time 6.0 user hit this". Fix
what the brief's fix policy allows, one SDD change package per coherent fix, tests first
(drill reproduces the bug, fix, drill goes red to prove it, invert to guard), spec and companions
updated, `fledge lanes run verify` green. Prefer one final PR; open a second only when a fix needs
its own spec ownership. Documentation errors are fixes too. Defer everything else with the issue
number and the reason in the report.

Phase 4, re-verify. Rebuild the binary from the fix branch and rerun every drill that failed
plus a random third of the ones that passed. Then run the confidence scoring exactly as the brief
defines it: two independent verifiers and one refuter per checklist claim, weighted P1 = 3 and
P2 = 1. Write `docs/6-0-confidence-report.md` with the score, every claim's evidence (command,
output, SHA), the list of deferred items, and a "what could still force a 7.0" section carried
forward from `docs/6-0-release-review.md`. If the score is below 95 percent or any P1 is open, go
back to Phase 3.

Phase 5, hand off. Land the final PR through the full lifecycle (approve, `check --commit`, push,
CI green, address Codex and corvid-agent threads, resolve them, review, ship, archive tip, CI
green). Close the issues the brief lists as already fixed on main, each with a comment naming
the commit or PR, plus any you fixed tonight. End the report with the exact human steps that
remain, copied from `docs/RELEASING.md`: cut the next `-rc.N` from the merged tree, watch
qualification, dispatch `promote`, `cargo publish`, Homebrew bump, changelog date, `v6` floating
tag. Your last message must stand on its own: score, what you fixed, what you deferred, what the
human does next, and anything you are unsure about.

If you hit a permission block, do not retry the same call; find a legitimate alternative or write
the blocked step into the handoff. If CI is red, read the failing job log before touching
anything. If corvid-agent blocks on a stale tip, address, comment, re-request, and continue; do
not wait on it. Do not stop because the session is long.
