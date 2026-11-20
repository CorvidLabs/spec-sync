---
kind: session
id: 2026-11-20-release-6-4-0-prep
aliases: [release-6-4-0-prep]
objective: "SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish"
intent_scope: ""
modules: []
stage: plan
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
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: "Build: red first on the candidate test's anchors, then the version, CHANGELOG and release notes"
---
# SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish

## Goal

SpecSync 6.4.0 release preparation on main: version 6.4.0 in every place RELEASING.md and validate-release-version.py name, a 6.4.0 CHANGELOG section and docs/releases/6.4.0.md, as a PR that waits for Leif's review and his yes; no tag, release or publish

## Where it stands

## Decisions and why

- Authorization: ss-lead (hc t58) assigned the 6.4.0 release-prep PR on 2026-11-20, from its release plan that Leif's answers set (relayed by hc t4c, 2026-11-19 about 15:49Z and 15:53Z). Leif's definition approval, review and yes to merge are his own; no tag, release, publish or workflow dispatch here.
- decided: follow #1068's 6.3.0 shape (operations, no spec change, the same surfaces), because validate-release-version.py at Cargo 6.4.0 asks for exactly those: ci.yml's packaged Action consumer, trust.yml's candidate, README and site Action examples, the CHANGELOG section and its Unreleased link
- rejected: moving the action.yml default, PUBLISHED_ACTION_DEFAULT, the Action docs default row or current-install guidance, because the validator requires them at published 6.3.0 until after publication
- Binary: every specsync command this session records runs main's release binary, built by cargo from tree 884d3eba (main 10d9bb3f's tree, #1100's head 5e962cad) and copied to this clone's target/release, first on PATH. The change itself was created and answered once with Homebrew's released 6.3.0, which PATH found first; main's binary recreated it in a throwaway clone and wrote the same records.
- decided: CHANGELOG heading ## [6.4.0] - 2026-11-24, the planned tag date; the date is corrected before the tag if the day moves (ss-lead t58, 2026-11-20)
- 0xLeif decided "Leif's definition approval of the 6.4.0 preparation (relayed by hc t4c through ss-lead t58)": "yes" (Claude Code chat message to his planning session (hc t4c), 2026-11-20 about 01:11Z, answering 'Approve the prep PR as written?' on the frozen ss-lead/gates/640-prep-definition.md, including the reworded date sentence and the measured correction of #1086's CHANGELOG sentence inside the approved scope; relayed by hc t4c through ss-lead (hc t58)); recorded by claude-code-t59; a relayed claim, not authenticated; ran specsync change approve prepare-specsync-6-4-0-package-metadata-and-release-handoff as 0xLeif

## What was verified

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
