---
kind: session
id: 2026-11-19-1084-history-fragments
aliases: [1084-history-fragments]
objective: At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)
modules: [change]
stage: done
stage_status: archived
iteration: 1
native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
criteria: [MERGE-1]
observations: [2026-11-19-covers-req-change-159-160-req-validator-048-req-hash-cache-006]
closed: 2026-11-19
outcome: done
---
# At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

## Goal

At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c)

## Where it stands

- Change opened with six deltas and its artifacts; failing tests in tests/integration/spec_history.rs fail on 65864de8 (7 of 8: no history subcommand, materialization bumps version:, check ignores fragments)
- Implemented REQ-change-159/160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037, REQ-cli-args-026, REQ-cli-014: fragment at materialization, legacy row still counts, fragment in commit scope/coverage/ownership/verification-commit paths/send-back, check validation and cache binding, specsync history; tests updated for no bump/row; specs, change.3md z=53, site docs, AGENTS.md, CHANGELOG
- change check --commit (d49f4a3b, d2fa5b6b), verify lane on rust 1.89.0, change audit --strict, fledge trust verify all passed

## Decisions and why

- Authorization: Leif approved spec-sync #1084 on 2026-11-18, written on the issue as 'Leif's decision (2026-11-18): approved', relayed to this agent by hc t4c as his scope approval; scope is the issue's three bullets; limits: no release, tag or publish, no merge without his relayed yes
- Intent: MERGE-1 already says two changes to one module merge cleanly with no conflict on version: or the change-log row; #1084 makes that hold on GitHub's merge button, which runs no merge driver. No new HI criterion is captured because none was confirmed
- decided: the freeze needs no command and rewrites nothing, because the issue says no row is rewritten and a per-spec marker would itself touch every spec; rejected: converting rows into fragments
- 0xLeif decided "Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump as 0xLeif
- Delta corrected before any code was checked: it now also MODIFIES REQ-change-021, REQ-change-092 and REQ-change-123, which still described a Change Log row and a version bump that the issue removes; same scope, so re-approved under the same relayed authority
- 0xLeif decided "Scope re-approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires); recorded by claude-code-t4t; a relayed claim, not authenticated
- 0xLeif decided "Scope re-approval at plan of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump": "Leif's decision (2026-11-18): approved." (Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump as 0xLeif
- decided: a module owns every .md file in its history/ directory for coverage and acceptance ownership, as it owns its companions, because a successor's manifest can name a predecessor's fragment; commit staging stays limited to the change's own fragment
- Environment note: this Mac's global git config sets tag.gpgsign, so changelog::tests' lightweight git tag fails; they pass with GIT_CONFIG_GLOBAL=/dev/null and do not touch this change
- 0xLeif decided "Leif's review of #1084 (PR #1086)": "Approve, merge when green" (Leif's implementation review of PR #1086 on 2026-11-18, after the tester's PASS (prs/ss1086/notes.md); relayed by his planning session hc t4c); recorded by claude-code-t4t; a relayed claim, not authenticated; ran specsync change review record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump as 0xLeif with verdict pass

## What was verified

- Covers REQ-change-159, -160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037 (the last two are not linked: chough 0.6.0 looks them up under specs/hash-cache and specs/cmd-change). Failing first on 65864de8: 7 of 8 tests in tests/integration/spec_history.rs failed, and main's release binary made two branches (ADDED REQ-auth-002; MODIFIED Invariants) that conflict under plain git merge in specs/auth/auth.spec.md on the two Change Log rows. Now, with GIT_CONFIG_GLOBAL=/dev/null (this Mac's global tag.gpgsign breaks changelog::tests' lightweight tags; they do not touch this change): fledge lanes run verify passed (fmt, clippy --all-targets -D warnings, check, 3077 unit and 627 integration tests, release build, specsync check --strict --require-coverage 100 62/62, CI script tests). The same two branches merge with plain git, no driver; version: and the Change Log stay byte for byte; both fragments are kept; specsync history renders them. This change's own change check --commit wrote six fragments and bumped no version. change audit --strict and fledge trust verify (augur review, risk 39) passed. ^2026-11-19-covers-req-change-159-160-req-validator-048-req-hash-cache-006


## Bugs

None.

## Open threads

## Timeline

- 2026-11-19T00:29:41Z · start · intent → intent · At ship SpecSync writes specs/<module>/history/<change-id>.md instead of a Change Log row and a version bump; check validates the fragments; specsync history renders the table; existing tables are frozen as written, no row rewritten (#1084, Leif's approval on the issue, relayed by hc t4c) ^t-0a145b
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - by: claude-code (hc t4t)
- 2026-11-19T00:29:51Z · advance · intent → design ·  ^t-2c39e2
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - by: claude-code (hc t4t)
- 2026-11-19T00:29:51Z · skip · design → plan · CLI and record-format change only: no visuals or interactions ^t-478550
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - by: claude-code (hc t4t)
- 2026-11-19T00:38:22Z · gate-opened · plan → plan · Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-c7128a
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T00:38:27Z · gate-closed · plan → plan · Scope approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-2f08e1
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: 65864de8194c10a26f674a7a171c500ae7ba5792
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1084 on 2026-11-18; relayed to this agent by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets
    - recorded_by: claude-code-t4t
    - native: specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T00:38:34Z · advance · plan → build · native:approved ^t-27e158
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:approved
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:21Z · gate-opened · build → build · Scope re-approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-c065e2
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - holds: build
  - drives: none
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:21Z · gate-closed · build → build · Scope re-approval of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-00e274
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires
    - recorded_by: claude-code-t4t
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:40Z · back · build → plan · The delta gained MODIFIED REQ-change-021, -092 and -123; the build-stage gate drove no native approval, so the re-approval is recorded at plan ^t-847f37
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - refresh: plan: native definition; scope or contract changes need native reapproval, build: implementation against the current definition, native: the definition is approved; changing intent, contract, acceptance criteria or scope needs native reapproval. Follow `specsync change status record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump`; Chough does not change native state.
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:40Z · gate-opened · plan → plan · Scope re-approval at plan of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-a76902
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - holds: plan
  - drives: approve
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:55Z · gate-closed · plan → plan · Scope re-approval at plan of record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump ^t-8e7534
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - relayed_by:
    - decided_by: 0xLeif
    - words: Leif's decision (2026-11-18): approved.
    - where: Written on spec-sync issue #1084 on 2026-11-18; relayed by hc t4c on 2026-11-18 as his scope approval of the issue's three bullets; re-recorded after the delta added MODIFIED REQ-change-021, -092 and -123, which that scope requires
    - recorded_by: claude-code-t4t
    - native: specsync change approve record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T00:45:59Z · advance · plan → build · native:approved ^t-1c41ef
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:approved
  - revision: bc67639d587286bc1024220a7b5e61c7db2430cd
  - by: claude-code (hc t4t)
- 2026-11-19T01:29:19Z · advance · build → test · native:verifying ^t-6a9226
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:verifying
  - revision: d2fa5b6bc8686320333be8463b0d2fa913d836d5
  - by: claude-code (hc t4t)
- 2026-11-19T01:31:01Z · advance · test → accept · native:verifying ^t-b22054
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:verifying
  - revision: b2dcb5a54e59666cc98dcc91f3996dd5e96533f2
  - by: claude-code (hc t4t)
- 2026-11-19T01:31:01Z · gate-opened · accept → accept · Leif's review of #1084 (PR #1086) ^t-467e70
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: b2dcb5a54e59666cc98dcc91f3996dd5e96533f2
  - holds: accept
  - drives: review
  - by: claude-code (hc t4t)
- 2026-11-19T01:46:33Z · gate-closed · accept → accept · Leif's review of #1084 (PR #1086) ^t-4221b3
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - revision: 34abea1031322cda830c79d199123dbba85c8053
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve, merge when green
    - where: Leif's implementation review of PR #1086 on 2026-11-18, after the tester's PASS (prs/ss1086/notes.md); relayed by his planning session hc t4c
    - recorded_by: claude-code-t4t
    - native: specsync change review record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
    - actor: 0xLeif
  - by: claude-code (hc t4t)
- 2026-11-19T01:46:35Z · advance · accept → done · native:archived ^t-d90277
  - native_change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
  - gate: native:archived
  - revision: 34abea1031322cda830c79d199123dbba85c8053
  - by: claude-code (hc t4t)

Closed 2026-11-19: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-19-covers-req-change-159-160-req-validator-048-req-hash-cache-006
    claim: "Covers REQ-change-159, -160, REQ-validator-048, REQ-hash-cache-006, REQ-cmd-change-037 (the last two are not linked: chough 0.6.0 looks them up under specs/hash-cache and specs/cmd-change). Failing first on 65864de8: 7 of 8 tests in tests/integration/spec_history.rs failed, and main's release binary made two branches (ADDED REQ-auth-002; MODIFIED Invariants) that conflict under plain git merge in specs/auth/auth.spec.md on the two Change Log rows. Now, with GIT_CONFIG_GLOBAL=/dev/null (this Mac's global tag.gpgsign breaks changelog::tests' lightweight tags; they do not touch this change): fledge lanes run verify passed (fmt, clippy --all-targets -D warnings, check, 3077 unit and 627 integration tests, release build, specsync check --strict --require-coverage 100 62/62, CI script tests). The same two branches merge with plain git, no driver; version: and the Change Log stay byte for byte; both fragments are kept; specsync history renders them. This change's own change check --commit wrote six fragments and bumped no version. change audit --strict and fledge trust verify (augur review, risk 39) passed."
    result: passed
    revision: d2fa5b6bc8686320333be8463b0d2fa913d836d5
    environment: local macOS 26, rust 1.89.0 (rust-toolchain.toml), branch 1084-history-fragments at d2fa5b6b
    content: aad27e38a54cc3cf876bb6b7a8059ae041e8b7a242e2864f515b6b1a41e2089f
    observed: 2026-11-19T01:30:14Z
    actor: claude-code (hc t4t)
```
