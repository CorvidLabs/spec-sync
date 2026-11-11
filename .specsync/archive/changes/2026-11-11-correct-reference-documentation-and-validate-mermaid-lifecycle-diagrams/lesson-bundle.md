# Lesson bundle: correct-reference-documentation-and-validate-mermaid-lifecycle-diagrams

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Correct reference documentation and validate Mermaid lifecycle diagrams
- **Kind**: Documentation
- **Specs**: agents, github
- **Paths**: hi/docs.md, INTENT.md, README.md, CONTRIBUTING.md, docs/HLD.md, docs/ADOPTING.md, docs/RELEASING.md, docs/ci-confidence.md, site/README.md, site/src/content/docs/cli.md, site/src/content/docs/configuration.md, site/src/content/docs/architecture.md, site/src/content/docs/companion-files.md, site/src/content/docs/cross-project-refs.md, site/src/content/docs/quickstart.md, site/src/content/docs/integrations/github-action.md, site/src/content/docs/integrations/vscode-extension.md, site/src/content/docs/comparisons/spec-kit.md, site/src/content/docs/comparisons/adversarial-proof.md, site/src/content/docs/why-specsync.md, site/package.json, site/bun.lock, site/scripts/documentation-accuracy.test.ts, src/agents.rs, .specsync/agent-artifacts.json, .claude/skills/spec-sync/SKILL.md, .codex/skills/spec-sync/SKILL.md, .cursor/skills/spec-sync/SKILL.md, .gemini/skills/spec-sync/SKILL.md, .agents/skills/chough-workflow/references/lifecycle.md, .agents/skills/chough-workflow/references/accept.md, .chough/, specs/agents/agents.spec.md, specs/agents/requirements.md, specs/agents/tasks.md, specs/agents/testing.md, specs/agents/agents.3md, specs/github/github.spec.md, specs/github/requirements.md, specs/github/tasks.md, specs/github/testing.md, specs/github/github.3md
- **Acceptance**: DOCS-4: Reference and contributor documentation accurately describes current commands, settings, defaults, paths, supported platforms and links.
- **Acceptance**: DOCS-5: All maintained diagrams reflect current implementation states, transitions and steps.
- **Acceptance**: DOCS-5.a: CI checks every maintained Mermaid block for parser validity and checks lifecycle states and send-back transitions against source, with failing mutation controls.
- **Acceptance**: DOCS-2 and DOCS-2.a: Reference and generated/copied skill closeout guidance agrees on local verify, review, native archive commit, push and CI.
- **Acceptance**: Protected open-PR lines remain unchanged; HLD F22 follows #935 and Chough session records follow #941. One docs-focused PR carries tests, generated prose and contracts without changing executable lifecycle behavior.

## Evidence

- Verification commit: `bb88f200b0b4d6ee71b46149e4509e949285a2c0`
- Base commit: `7ae7245f97e7cfd66cd80ce39b611becb1095642`
- Verified by: `specsync check --spec agents --spec cmd_change --spec github`

## From the change's context.md

# Context

## Human authority

Leif's task was to make the README, docs and Mermaid diagrams accurate. A read-only audit produced 22 findings in a local, uncommitted findings report, pinned originally to e6b1ce11. t18 received that report before fixes. On 2026-11-06 t18 relayed Leif's exact decision, “Capture all three”, approving DOCS-4, DOCS-5 and DOCS-5.a as proposed. Codex captured the exact sentences with fledge hi and recorded that relay in hi/docs.md. This is a relayed claim, not authenticated identity.

Scope approval is pending: t18 explicitly requested `ready for approval <id> at <commit>` and will record Leif's approval. No native approval, test implementation, prose fix, push or PR is authorized by an agent label. Later implementation acceptance remains separate.

## Evidence and baseline

Preparation base is main 7ae7245f97e7cfd66cd80ce39b611becb1095642. The branch was clean before fast-forward. Fledge built this checkout in Herdr t130. Audit commands used the separately built e6b1ce11 binary (hash and exact receipts kept locally, outside the repository). The audit covered 71 visible help pages, hidden accept help, selected disposable command/lifecycle fixtures, 51 external links and every Mermaid fence. All 11 diagrams were reviewed manually; offline CLI parser unavailable. No parser pass, full suite pass, deployment or cross-platform parity is claimed from that audit.

## Decisions and constraints

Use one documentation-focused change; maintain the agents and github contracts through deltas. Generated skill text is corrected at its template source and regenerated, never hand-edited only in installed copies. Add parser and source-backed semantic checks to the already-required site test lane instead of duplicating README/landing workflow harnesses owned elsewhere. Test the current bad prose/diagram first and retain the red result. Current source determines facts; historical release records stay historical. No network link monitor is added to routine CI.

## Workflow recovery dependency

Chough next refuses several sibling sessions without an active pointer. Chough session new docs-accuracy refuses `.chough/session.json exists; run chough session adopt <key> first`. t18 prohibited adopting/moving those records and assigned recovery to ss-linux-arm64/#941. No sibling checkpoint has been modified. Native session creation and this change's Chough checkpoint wait for #941. Only this session's new records and evidence may be created under the declared .chough/ prefix.

## Open dependencies

#935 edits the exact HLD inherited-context row. t18 assigned its additional lifecycle omission (F22) here, but the correction waits for #935 to land; no overlapping line is edited while it is open. Its ss-contracts follow-up issue number is pending. The protected-hunk inventory and complete intended scope are in plan.md. No other agent was spawned.

## From the change's design.md

# Documentation validation design

## Architecture

Add site/scripts/documentation-accuracy.test.ts to the existing Bun test discovery. It reads maintained Markdown/MDX/Astro documents from the repository and extracts all Mermaid blocks, reporting file/line on error. It uses the official Mermaid parser with pinned development dependencies; no browser rendering, external service or network request is part of normal execution. The parser API is documented at https://mermaid.js.org/config/usage.html#syntax-validation-without-rendering.

A separate assertion reads ChangeState and SEND_BACK_STATES from the Rust source and compares the HLD state diagram's nodes and send-back edges. Unknown or unreadable source structure must fail clearly rather than silently dropping expected states. Fixtures deliberately corrupt syntax/remove an edge/add a phantom state to prove rejection. Closeout sequence/staging assertions are tied to explicit code behavior and retained runtime fixture receipts; syntax success alone is not semantic correctness.

## Generated guidance

Edit only skill prose, its version metadata and focused tests in src/agents.rs. Increment the template version as the module lesson requires. Regenerate tracked integrations and the digest manifest with the current built binary. Preserve all installer safety, idempotency and custom-file protection. Modify the matching requirement so docs, template and tests agree about the automatic archive commit.

## Scope boundaries

No CLI behavior, runtime guard, supported-platform matrix, workflow selection, human gate, publication operation or timeout is changed. HLD diagrams remain explanatory views of existing behavior, not new behavior definitions. There is no product UI interaction or visual redesign to prototype.

## Failure behavior

Mermaid/parser failures, missing lifecycle nodes/edges and invalid source extracts fail the test with a location. Dependency installation failures are reported as environment failures, not parser passes. Test-only dependencies are locked; public external links are checked manually with bounded commands and recorded separately from CI.

## From the change's testing.md

# Testing plan

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-agents-009 | `agents::tests::tracked_agent_files_teach_archive_first_ci`, `tracked_agent_artifacts_and_manifest_match_a_fresh_install`, `reinstall_keeps_all_generated_artifacts_byte_identical`, and the customization-preservation tests in `cargo test agents::tests::`; red commit `355f1d69`, final run 45 passed / 0 failed |
| REQ-github-027 | `site/scripts/documentation-accuracy.test.ts`: all 11 maintained fences parse, exact ChangeState/SEND_BACK_STATES correspondence, malformed/missing/phantom-state/edge/source-shape mutation controls; 25 passed / 0 failed on the corrected docs. The existing site lane discovers it: 48 passed / 0 failed on Bun 1.3.14 |

## Observed implementation checks

- Red commit `355f1d69`: 21/25 documentation checks passed, four failed; the exact generated
  skill regression ran once and failed. The raw logs were removed on 2026-11-11 because they
  held local home-directory paths; commit `355f1d69` still reproduces both failures.
- Final focused runs: 25/25 documentation checks and 45/45 agent tests passed. Full site suite:
  48/48, Astro 0 errors / 0 warnings, 43-page build; pinned Bun 1.3.14 frozen install succeeded.
- The documented local VSIX workflow installed frozen dependencies, compiled and packaged
  `specsync-1.0.0.vsix` successfully. No Marketplace install, publication or editor UI is claimed.
- All three replacement primary-source URLs returned HTTP 200 on 2026-11-06. The Marketplace
  404 in the original audit is retained; current crates.io probing returned HTTP 403, so the
  corrected runbook makes no unverified claim about its latest version.
- Protected #900 README/CLI, #918 CLI timeout, and #952 landing/test hunks were re-read and
  remain untouched. F22 follows merged #935; legacy/sibling Chough recovery remains with #941.
- Native materialization succeeded, then verification refused missing requirement evidence;
  this table supplies it. Full verify, trust, audit/pre-push and human implementation review
  remain separate gates, not inferred from the focused passes above.

## Failing first

1. Add the focused documentation test before prose/diagram changes. On current HLD, require a failure for missing send-back edges and stale staging/archive ordering. Add malformed Mermaid and lifecycle drift mutation controls; require them to fail independently of the real documents.
2. Extend the generated-skill archive-first test before editing SKILL_BODY: require native ship/finalize to be described as creating the archive commit. Retain its failure, then update the text and regenerate all copies.
3. Use the same assertions after fixes; never loosen a bound, skip, expectation or timeout to obtain green.

## Focused verification

- In site/: `bun test scripts/documentation-accuracy.test.ts`, followed by `bun test`, `bun run lint`, `bun run build` with the frozen lockfile dependencies.
- `cargo test agents::` for installer/generation/parity; use a Herdr terminal for build/test work.
- Repository-pinned `specsync check --strict` and `specsync score`; fix affected findings and preserve unrelated evidence.
- `python3 -S .github/scripts/validate-release-version.py` and `python3 -S .github/scripts/validate-workflow-runtime-pins.py` for edited version/install prose.
- Bounded manual primary-source link probes and documented command fixtures as needed for changed claims; report HTTP reachability separately from claim correctness.
- Inspect current PR diffs and the local changed-line/path set before review. Confirm no protected hunk or sibling Chough record changed.

## Required delivery gates

Run fledge lanes run verify and fledge trust verify before completion; use the configured Attest/Augur flow and surface a block. Before every push also run fledge lanes run pre-push and repository-pinned `specsync change audit --strict` (t18's explicit instruction after #900). The latter is required even when local check --commit and verify passed.

Review and ship are sequential with no intervening commit. Wait for hosted CI only after the archive tip is pushed. Human scope approval, implementation acceptance and GitHub merge are separate. No tag/release/publication/force push is part of the test plan.

## Existing audit limits

Original pinned build and selected lifecycle fixtures passed. All 11 diagrams were manually reviewed; offline Mermaid CLI was unavailable. Four external URLs returned 404. The release-candidate validator suite hit the audit runner's fixed 45-second limit and was not retried or declared passed. Those historical results do not stand in for checks on this future implementation.

## Full local lane result

`PATH=<rustup-cargo-bin>:<pinned-Bun-directory>:$PATH fledge lanes run verify` passed
all seven configured steps in 18m53s. Rust/Clippy were 1.89.0, as pinned in
`rust-toolchain.toml`: 2838 unit tests passed, 2 ignored; all 499 integration tests passed;
release build, strict spec check, release-candidate, CI-gate and Linux-baseline guard tests
passed. The raw log was removed on 2026-11-11 (it held local home-directory paths).
The run started at 3bfa1bdd; the only commit during it was 21c732a4 (contributor prose and
diagnostic receipts, no Rust/test/configuration changes). Chough case capture remains blocked
by #941, which owns sibling checkpoint recovery. No human implementation acceptance, archived
PR CI, merge or release is inferred from this local result.

## Update after merging main (2026-11-11)

Main was merged in at `85804be2` and the corrections in `c6368fae` followed: the HLD lifecycle
diagram gained `Retired` and the `change retire` edge (#1018), `require_draft_files` moved to
the top-level settings, the Spec Kit persistence link moved, the Windows and Trust guidance now
states what ships and what SpecSync's own CI pins, and the raw `.chough` logs (local
home-directory paths) and the hand-edited Chough skill copy were dropped. The sections above
describe the 2026-11-06 head and stay as history.

Run at `c6368fae` on macOS arm64 (rustc 1.89.0, Bun 1.4.0):

- `bun test scripts/documentation-accuracy.test.ts`: 25 passed, 0 failed. All 11 maintained
  Mermaid diagrams parse; HLD states match `ChangeState`, including `Retired`; send-back edges
  match `SEND_BACK_STATES`. Before the `Retired` fix, the same test on a trial merge failed
  exactly the state-correspondence check.
- `cargo test --bin specsync agents::tests::`: 45 passed (REQ-agents-009).
- `cargo test --test integration documented_workflows`: 4 passed.
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: clean. `hi check`: clean.

The full `cargo test`, verify lane and site build run in hosted CI at the pushed head
(coordinator rule, 2026-11-11); nothing merges unless it is green.

## Where these lessons go

- `specs/agents/agents.3md`
- `specs/github/github.3md`
