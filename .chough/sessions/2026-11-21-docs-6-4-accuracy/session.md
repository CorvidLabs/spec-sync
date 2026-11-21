---
kind: session
id: 2026-11-21-docs-6-4-accuracy
aliases: [docs-6-4-accuracy]
objective: "The README and every current documentation page, Mermaid diagrams included, say what SpecSync 6.4.0 and 6.4.1 do: the 21 items in defs/docs-6.4-accuracy-definition.md, documentation only, no version or pin wording (the 6.4.1 post-publication PR owns it); scope confirmed by t4c under Leif's standing OK for scope approvals"
intent_scope: CHANGE
modules: []
stage: done
stage_status: active
iteration: 1
callers: []
native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
pr: "1141"
criteria: [DOCS-1, DOCS-2, DOCS-4, DOCS-5]
requirements: []
cases: []
design: []
files: []
observations: [2026-11-21-documentation-only-the-21-items-of-defs-docs-6-4-accuracy]
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Continue done
---
# The README and every current documentation page, Mermaid diagrams included, say what SpecSync 6.4.0 and 6.4.1 do: the 21 items in defs/docs-6.4-accuracy-definition.md, documentation only, no version or pin wording (the 6.4.1 post-publication PR owns it); scope confirmed by t4c under Leif's standing OK for scope approvals

## Goal

The README and every current documentation page, Mermaid diagrams included, say what SpecSync 6.4.0 and 6.4.1 do: the 21 items in defs/docs-6.4-accuracy-definition.md, documentation only, no version or pin wording (the 6.4.1 post-publication PR owns it); scope confirmed by t4c under Leif's standing OK for scope approvals

## Where it stands

## Decisions and why

- No intent criterion: a documentation-only change that alters no behavior; the confirmed definition proposes none.
- 0xLeif decided "Scope approval of correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples": "Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)" (Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'), confirmed for this definition (defs/docs-6.4-accuracy-definition.md: documentation only, --no-spec-change, no requirement wording, no refusal, the recommended fix for all 21 items) by t4c on 2026-11-21, sent directly from hc t4c while ss-lead t58 was stopped); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples as 0xLeif
- Correction to the intent-stage note: no new criterion, but existing ones cover this change. DOCS-4 (reference documentation accurate for the current version) and DOCS-5 (diagrams show the states and steps SpecSync uses) are its subject; DOCS-1 (README examples run as written) and DOCS-2 (the documented closeout is finished locally before push) cover items 7, 8 and 17 to 19.
- 0xLeif decided "Leif's review of the 6.4 documentation corrections": "Pass (Recommended)" (a Claude Code question dialog in his planning session hc t4c, on 2026-11-21 at about 15:25Z, to his review of #1141 (the 6.4 documentation PR, put to him from defs/1141-review-request.md at head 8d45da4c); relayed by t4c); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change review correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples as 0xLeif with verdict pass

## What was verified

- [Documentation only, the 21 items of defs/docs-6.4-accuracy-definition.md (audit of main 3f0985e8 with its 6.4.1 build). Command check: every specsync invocation the docs show is valid against --help (80 command paths; only prose or placeholder hits left; the four check --commit hits gone). New samples are real output of the 6.4.1 build: resolve --remote against spec-sync's registry (exit 1), a three-crate Cargo workspace before and after the fixes, init in an empty project. site: bun test 48/48 (Mermaid parse, lifecycle against the code), bun run build 43 pages, lint 0 errors; validate-release-version.py passes; examples sdd-lifecycle, sdd-five-epics, sdd-concurrent-changes pass. Lane at cc4977b2 (3158 unit, 741 integration, 62/62, 100%); change audit --strict exit 0.](evidence/2026-11-21-documentation-only-the-21-items-of-defs-docs-6-4-accuracy.md)

## Open threads

## Timeline

- 2026-11-21T00:53:55Z · start · intent → intent · The README and every current documentation page, Mermaid diagrams included, say what SpecSync 6.4.0 and 6.4.1 do: the 21 items in defs/docs-6.4-accuracy-definition.md, documentation only, no version or pin wording (the 6.4.1 post-publication PR owns it); scope confirmed by t4c under Leif's standing OK for scope approvals ^t-309c35
  - revision: 3f0985e80acab0fe4559fe322cf929be40057414
  - by: claude-code (hc t81)
- 2026-11-21T00:54:30Z · direct-native-write · intent → intent · correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-8d82ef
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 3f0985e80acab0fe4559fe322cf929be40057414
  - direct_write:
    - change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t81)
- 2026-11-21T00:54:30Z · advance · intent → design ·  ^t-a25860
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 3f0985e80acab0fe4559fe322cf929be40057414
  - by: claude-code (hc t81)
- 2026-11-21T00:54:30Z · advance · design → plan ·  ^t-11477d
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 3f0985e80acab0fe4559fe322cf929be40057414
  - by: claude-code (hc t81)
- 2026-11-21T00:54:30Z · gate-opened · plan → plan · Scope approval of correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples ^t-906b5b
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 3f0985e80acab0fe4559fe322cf929be40057414
  - holds: plan
  - drives: approve
  - by: claude-code (hc t81)
- 2026-11-21T00:54:30Z · gate-closed · plan → plan · Scope approval of correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples ^t-a59cbd
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 3f0985e80acab0fe4559fe322cf929be40057414
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's standing OK for scope approvals (2026-11-20 dialog, relayed via t4c)
    - where: Leif's standing OK for scope approvals (2026-11-20 dialog, 'Standing OK, tell me after'), confirmed for this definition (defs/docs-6.4-accuracy-definition.md: documentation only, --no-spec-change, no requirement wording, no refusal, the recommended fix for all 21 items) by t4c on 2026-11-21, sent directly from hc t4c while ss-lead t58 was stopped
    - recorded_by: claude-code-t4t
    - native: specsync change approve correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
    - actor: 0xLeif
  - by: claude-code (hc t81)
- 2026-11-21T01:11:00Z · direct-native-write · plan → plan · correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples's lifecycle records changed outside Chough: state.json changed, verification.json added, verification-attempts.json added; state approved → verifying ^t-5d22e2
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 590adc48cd186c9ad145bb239de3c6110fdba8b3
  - direct_write:
    - change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
    - files:
      - state.json: changed
      - verification.json: added
      - verification-attempts.json: added
    - before_state: approved
    - after_state: verifying
  - by: claude-code (hc t81)
- 2026-11-21T01:11:00Z · advance · plan → build · native:verifying ^t-c3bcc7
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - gate: native:verifying
  - revision: 590adc48cd186c9ad145bb239de3c6110fdba8b3
  - by: claude-code (hc t81)
- 2026-11-21T01:11:02Z · advance · build → test · native:verifying ^t-73e140
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - gate: native:verifying
  - revision: 590adc48cd186c9ad145bb239de3c6110fdba8b3
  - by: claude-code (hc t81)
- 2026-11-21T01:11:04Z · advance · test → accept · native:verifying ^t-64788d
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - gate: native:verifying
  - revision: 590adc48cd186c9ad145bb239de3c6110fdba8b3
  - by: claude-code (hc t81)
- 2026-11-21T01:11:04Z · gate-opened · accept → accept · Leif's review of the 6.4 documentation corrections ^t-c26453
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 590adc48cd186c9ad145bb239de3c6110fdba8b3
  - holds: accept
  - drives: review
  - by: claude-code (hc t81)
- 2026-11-21T15:25:45Z · direct-native-write · accept → accept · correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples's lifecycle records changed outside Chough: state.json changed, verification.json changed, verification-attempts.json changed; state stays verifying ^t-ccc2d0
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 8d45da4c1a2dd53294f4252c6fa1fd60d508f8cc
  - direct_write:
    - change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
    - files:
      - state.json: changed
      - verification.json: changed
      - verification-attempts.json: changed
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t81)
- 2026-11-21T15:25:45Z · gate-closed · accept → accept · Leif's review of the 6.4 documentation corrections ^t-00a421
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 8d45da4c1a2dd53294f4252c6fa1fd60d508f8cc
  - relayed_by:
    - decided_by: 0xLeif
    - words: Pass (Recommended)
    - where: a Claude Code question dialog in his planning session hc t4c, on 2026-11-21 at about 15:25Z, to his review of #1141 (the 6.4 documentation PR, put to him from defs/1141-review-request.md at head 8d45da4c); relayed by t4c
    - recorded_by: claude-code-t4t
    - native: specsync change review correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
    - actor: 0xLeif
  - by: claude-code (hc t81)
- 2026-11-21T15:29:37Z · direct-native-write · accept → accept · correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples's lifecycle records changed outside Chough: state.json removed, approvals.json removed, review.json removed, review-attempts.json removed, verification.json removed, verification-attempts.json removed; state verifying → unknown ^t-4eca86
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - revision: 6a1b4bb7194333bb9823164005de4d76f1dbd676
  - direct_write:
    - change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
    - files:
      - state.json: removed
      - approvals.json: removed
      - review.json: removed
      - review-attempts.json: removed
      - verification.json: removed
      - verification-attempts.json: removed
    - before_state: verifying
  - by: claude-code (hc t81)
- 2026-11-21T15:29:37Z · advance · accept → done · native:archived ^t-7ddecc
  - native_change: correct-the-readme-and-documentation-for-specsync-6-4-history-fragments-the-archive-commit-diagrams-and-output-samples
  - gate: native:archived
  - revision: 6a1b4bb7194333bb9823164005de4d76f1dbd676
  - by: claude-code (hc t81)
