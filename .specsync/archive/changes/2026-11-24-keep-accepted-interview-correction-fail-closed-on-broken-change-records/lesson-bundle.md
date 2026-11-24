# Lesson bundle: keep-accepted-interview-correction-fail-closed-on-broken-change-records

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep accepted interview correction fail-closed on broken change records
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, tests/integration/terminal_evidence_isolation.rs, specs/change/
- **Acceptance**: When an unrelated change has corrupt committed workflow history, correcting accepted interview metadata must fail with that history error before writing any new evidence. The same correction must succeed with a valid project roster, and the healthy change’s read-only status must remain independent from that sibling.

## Evidence

- Verification commit: `d1a41ea7d6e9cad067148b48a7cba838d36de684`
- Base commit: `16a43b93a83a9487dacc38109af993e3d38cd8a1`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

The terminal evidence isolation in PR #1114 uses a roster that sets aside per-change failures for read-only lifecycle summaries. The same tolerant helper was also called by `change correct`, which writes new approval, correction and verification records. Route that writer through the existing fully validated roster. Keep read-only isolation and other writers’ rules intact.

Review finding: https://github.com/CorvidLabs/spec-sync/pull/1114#discussion_r4674008149

## From the change's testing.md

# Testing

Use the public change lifecycle to archive a sibling, then produce a committed duplicate state and remove the active copy. Interrupt finalization of a valid target to obtain an accepted change, and verify that its read-only status succeeds. `change correct` must name the unrelated committed-history error and leave its workspace unchanged. Run the same correction from the healthy fixture and verify that it writes the correction ledger.

## Where these lessons go

- `specs/change/change.3md`
