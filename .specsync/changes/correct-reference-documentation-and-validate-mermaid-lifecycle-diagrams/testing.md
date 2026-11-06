# Testing plan

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
