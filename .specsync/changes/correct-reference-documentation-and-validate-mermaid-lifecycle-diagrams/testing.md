# Testing plan

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-agents-009 | `agents::tests::tracked_agent_files_teach_archive_first_ci`, `tracked_agent_artifacts_and_manifest_match_a_fresh_install`, `reinstall_keeps_all_generated_artifacts_byte_identical`, and the customization-preservation tests in `cargo test agents::tests::`; red commit `355f1d69`, final run 45 passed / 0 failed |
| REQ-github-027 | `site/scripts/documentation-accuracy.test.ts`: all 11 maintained fences parse, exact ChangeState/SEND_BACK_STATES correspondence, malformed/missing/phantom-state/edge/source-shape mutation controls; 25 passed / 0 failed on the corrected docs. The existing site lane discovers it: 48 passed / 0 failed on Bun 1.3.14 |

## Observed implementation checks

- Red commit `355f1d69`: 21/25 documentation checks passed, four failed; the exact generated
  skill regression ran once and failed. Retained in `.chough/artifacts/docs-accuracy-red-*.log`.
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
