---
kind: session
id: 2026-11-20-release-6-4-0-prep
aliases: [release-6-4-0-prep]
objective: "SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish"
intent_scope: ""
modules: []
stage: done
stage_status: active
iteration: 1
callers: []
native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
pr: ""
criteria: [CI-1.a, DOCS-4]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-20-640-red-candidate-anchors-at-9f8cea7e
  - 2026-11-20-640-released-6-3-0-against-this-build
  - 2026-11-20-640-verify-lane-at-7ff79527
  - 2026-11-20-640-release-validators-at-6-4-0
  - 2026-11-20-640-portable-fixture-rerun-at-140eedaf
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Continue done
---
# SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish

## Goal

SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish

## Where it stands

- Built 6.4.0 at 68b76f27 and checked at 7ff79527: version, candidate pins, Action examples, CHANGELOG [6.4.0] with the #1084 sentence corrected, docs/releases/6.4.0.md; lane 10/10, validators and change audit --strict pass

## Decisions and why

- Authorization: ss-lead (hc t58) assigned the 6.4.0 release-prep PR on 2026-11-20, from its release plan that Leif's answers set (relayed by hc t4c, 2026-11-19 about 15:49Z and 15:53Z). Leif's definition approval, review and yes to merge are his own; no tag, release, publish or workflow dispatch here.
- decided: follow #1068's 6.3.0 shape (operations, no spec change, the same surfaces), because validate-release-version.py at Cargo 6.4.0 asks for exactly those: ci.yml's packaged Action consumer, trust.yml's candidate, README and site Action examples, the CHANGELOG section and its Unreleased link
- rejected: moving the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row or current-install guidance, because the validator requires them at published 6.3.0 until after publication
- Binary: every specsync command this session records runs main's release binary, built by cargo from tree 884d3eba (main 10d9bb3f's tree, #1100's head 5e962cad) and copied to this clone's target/release, first on PATH. The change itself was created and answered once with Homebrew's released 6.3.0, which PATH found first; main's binary recreated it in a throwaway clone and wrote the same records.
- decided: CHANGELOG heading ## [6.4.0] - 2026-11-24, the planned tag date; the date is corrected before the tag if the day moves (ss-lead t58, 2026-11-20)
- 0xLeif decided "Leif's definition approval of the 6.4.0 preparation (relayed by hc t4c through ss-lead t58)": "yes" (Claude Code chat message to his planning session (hc t4c), 2026-11-20 about 01:11Z, answering 'Approve the prep PR as written?' on the frozen ss-lead/gates/640-prep-definition.md, including the reworded date sentence and the measured correction of #1086's CHANGELOG sentence inside the approved scope; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve prepare-specsync-6-4-0-package-metadata-and-release-handoff as 0xLeif
- How the release candidate test was run, so a reader gets the same result: GIT_CONFIG_GLOBAL=<a file holding only user.name, user.email, commit.gpgsign=false and tag.gpgsign=false> GIT_CONFIG_NOSYSTEM=1 python3 -I .github/scripts/test-validate-release-candidate.py < /dev/null, as a prefix on that one command and never exported. On a machine whose own Git configuration signs tags, two of its tests error 'fatal: no tag message?', because the script's git() helper does not isolate configuration; that is the same on main (#1110).
- 0xLeif decided "Leif's review of #1109 at a7d5c658 (relayed by hc t4c through ss-lead t58)": "Yes" (Claude Code chat message to his planning session (hc t4c), 2026-11-20 about 02:16Z, answering 'Does your review pass?' for spec-sync #1109 at head a7d5c658, on the frozen ss-lead/gates/640-prep-review-request.md; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change review prepare-specsync-6-4-0-package-metadata-and-release-handoff as 0xLeif with verdict pass

## What was verified

- [Red first at 9f8cea7e: with only test-validate-release-candidate.py's 19 anchors moved to 6.4.0, the test fails 7 tests, every one an anchor the unbumped surfaces do not have yet (Validated release version 6.4.0, ci.yml and trust.yml candidate versions, Cargo.lock, site Action refs). Run with python3 through one-lane, the neutral git config as a prefix only (#1092).](evidence/2026-11-20-640-red-candidate-anchors-at-9f8cea7e.md)
- [Homebrew's released specsync 6.3.0 against main 10d9bb3f's release build (tree 884d3eba, which printed 6.3.0 before the bump) on one PR-shaped fixture: 6.3.0 check passes at every tip; 6.3.0 change audit and audit --strict exit 1 on the branch that adds a history fragment, at the checked and the archive tip, and pass on main after the merge; 6.3.0 change check --commit on a change the new build recorded commits a Change Log row and version 2, the new check then exits 1 naming the double record, and removing the row (then restoring version:) makes check pass, after which change check --commit and audit --strict pass; approve --dry-run --json prints id, dry_run, would_approve and thirteen checks. The upgrade notes and the corrected #1084 CHANGELOG sentence quote this run.](evidence/2026-11-20-640-released-6-3-0-against-this-build.md)
- [At 7ff79527 (checked and committed), with this tree's own release build (specsync 6.4.0) first on PATH: fledge lanes run verify passed 10/10 steps in 11m31s through one-lane (unit 3100 passed, 2 ignored; integration 644; spec check 62/62 specs, 167/167 files; the release-script suites OK), the neutral git config as a prefix on the lane only (#1092). specsync change audit --strict against origin/main 10d9bb3f passes too.](evidence/2026-11-20-640-verify-lane-at-7ff79527.md)
- [At 68b76f27 and its checked head 7ff79527, the three release validators pass at Cargo 6.4.0: validate-release-version.py prints 'Validated release version 6.4.0 across current distribution surfaces', validate-workflow-runtime-pins.py passes, and test-validate-release-candidate.py runs 67 tests OK (neutral git config as a prefix only, #1092). The action.yml default, PUBLISHED_ACTION_DEFAULT and the docs default row stay at published 6.3.0, as the validator requires.](evidence/2026-11-20-640-release-validators-at-6-4-0.md)
- [One rerun of the fixture after its version restore stopped using BSD-only sed -i '' (it now writes through a temporary file, so GNU sed runs it too), with this branch's own release build (specsync 6.4.0, built at 68b76f27) against Homebrew's released 6.3.0: the same 26 commands and exit codes as the earlier run with main 10d9bb3f's build, which corvid-agent also matched on Linux. The corrected deltas.md sentence quotes this run.](evidence/2026-11-20-640-portable-fixture-rerun-at-140eedaf.md)

## Open threads

## Timeline

- 2026-11-20T00:56:54Z · start · intent → intent · SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish ^t-15de11
  - revision: 10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f
  - by: claude-code (hc t59)
- 2026-11-20T01:04:25Z · advance · intent → design ·  ^t-50d950
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - revision: 10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f
  - by: claude-code (hc t59)
- 2026-11-20T01:04:25Z · skip · design → plan · Release metadata and documentation only: no visuals or interactions ^t-070b73
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - revision: 10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f
  - by: claude-code (hc t59)
- 2026-11-20T01:04:37Z · gate-opened · plan → plan · Leif's definition approval of the 6.4.0 preparation (relayed by hc t4c through ss-lead t58) ^t-7581b8
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - revision: 10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f
  - holds: plan
  - drives: approve
  - by: claude-code (hc t59)
- 2026-11-20T01:12:02Z · gate-closed · plan → plan · Leif's definition approval of the 6.4.0 preparation (relayed by hc t4c through ss-lead t58) ^t-0439a5
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - revision: 10d9bb3fe27e56a11e4bbf00e6a6227efe647f5f
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes
    - where: Claude Code chat message to his planning session (hc t4c), 2026-11-20 about 01:11Z, answering 'Approve the prep PR as written?' on the frozen ss-lead/gates/640-prep-definition.md, including the reworded date sentence and the measured correction of #1086's CHANGELOG sentence inside the approved scope; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change approve prepare-specsync-6-4-0-package-metadata-and-release-handoff
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-20T01:12:16Z · advance · plan → build · native:approved ^t-6a3b47
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - gate: native:approved
  - revision: df249f61966b2ca84c0b1bc5e82904640a9743d6
  - by: claude-code (hc t59)
- 2026-11-20T02:17:17Z · advance · build → test · native:verifying ^t-9d4277
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - gate: native:verifying
  - revision: a7d5c658de4f3008a55fb5e29bc28267d4314934
  - by: claude-code (hc t59)
- 2026-11-20T02:17:20Z · advance · test → accept · native:verifying ^t-40a9a4
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - gate: native:verifying
  - revision: a7d5c658de4f3008a55fb5e29bc28267d4314934
  - by: claude-code (hc t59)
- 2026-11-20T02:17:25Z · gate-opened · accept → accept · Leif's review of #1109 at a7d5c658 (relayed by hc t4c through ss-lead t58) ^t-cca12d
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - revision: a7d5c658de4f3008a55fb5e29bc28267d4314934
  - holds: accept
  - drives: review
  - by: claude-code (hc t59)
- 2026-11-20T02:19:22Z · gate-closed · accept → accept · Leif's review of #1109 at a7d5c658 (relayed by hc t4c through ss-lead t58) ^t-c29b19
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - revision: 263a4c5b41600210f4cf760fdf5bdf50dc4983bc
  - relayed_by:
    - decided_by: 0xLeif
    - words: Yes
    - where: Claude Code chat message to his planning session (hc t4c), 2026-11-20 about 02:16Z, answering 'Does your review pass?' for spec-sync #1109 at head a7d5c658, on the frozen ss-lead/gates/640-prep-review-request.md; relayed by hc t4c through ss-lead (hc t58)
    - recorded_by: claude-code-t59
    - native: specsync change review prepare-specsync-6-4-0-package-metadata-and-release-handoff
    - actor: 0xLeif
  - by: claude-code (hc t59)
- 2026-11-20T02:19:24Z · advance · accept → done · native:archived ^t-f99681
  - native_change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
  - gate: native:archived
  - revision: 263a4c5b41600210f4cf760fdf5bdf50dc4983bc
  - by: claude-code (hc t59)
