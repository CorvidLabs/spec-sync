---
kind: session
id: 2026-11-19-repository-steered-paths
aliases: [repository-steered-paths]
objective: Writes, reads and caches a repository can steer stay inside its Git work tree, never reach the Git directory, and never hide a failure (F2-F9, R1, R2, G1 and point 6 of the independent review, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules:
  - util
  - change
  - cmd_check
  - hash_cache
  - config
  - validator
  - changelog
  - git_utils
  - cmd_import
  - cmd_init_registry
  - agents
  - mcp
  - hooks
  - commands
  - generator
  - merge
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
pr: ""
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-on-base-1b3974e9-with-only-the-new-tests-added-dfdaf4f6-22-of
  - 2026-11-19-with-the-fix-at-e60fbf8e-all-24-new-integration-tests-both-new
  - 2026-11-19-on-linux-at-e60fbf8e-change-5-a-clean-copy-in-a-local-rust-1
bugs: [2026-11-19-repository-steered-writes-reads-and-caches]
blockers: []
waiting_on_human:
  - gate: Implementation review of keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
    holds: accept
    drives: review
    opened: 2026-11-19T20:34:07Z
run_consents: []
next: Continue accept
---
# Writes, reads and caches a repository can steer stay inside its Git work tree, never reach the Git directory, and never hide a failure (F2-F9, R1, R2, G1 and point 6 of the independent review, private, with GHSA-7q5v-8jcc-rx54)

## Goal

Writes, reads and caches a repository can steer stay inside its Git work tree, never reach the Git directory, and never hide a failure (F2-F9, R1, R2, G1 and point 6 of the independent review, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Intent approval of proposed criterion CHECK-1.f": "yes" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 16:00Z, after t4c summed up security/change5-scope.md: SETUP-6 and SETUP-7 cover points 1, 2, 3 and 5; the new intent line CHECK-1.f 'A result that a repository somebody else wrote carries along never hides a failure my check would report.'; choices 1-8 as recommended (tracked hashes.json neither read nor saved with one warning and the git rm --cached fix; confinement root = git top folder else --root, caches strict, absolute links refused with an 'absolute link' message; confine specs_dir, source_dirs and schema_dir with a budgeted confined walk; a changelog blob over 8 MiB skipped with a warning; import --from-dir confined to the named directory, 8 MiB; --fix --backup keeps its layout, written confined; the wider write scan; the two low items included); relayed by t4c over hc to t5r, to be recorded when change 5 starts after change 4); recorded by claude-code-t5r; a relayed claim, not authenticated
- 0xLeif decided "Scope approval of point 6 for keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree: refuse paths that resolve into the Git directory": "All as recommended" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 19:43Z, after t4c gave him one table of recommendations whose second row was point 6 of change 5 as proposed: the project read and write helpers refuse a path that resolves into the Git directory, directly or through links (canonical target under git rev-parse --absolute-git-dir, or any .git component, any case, trailing dots and spaces), with the message '<path> leads into the Git directory', and SpecSync's own .git writes keep their own paths; relayed by t4c over hc to t5r as part of change 5's scope); recorded by claude-code-t5r; a relayed claim, not authenticated
- 0xLeif decided "Scope approval of keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree": "yes" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 16:00Z, after t4c summed up security/change5-scope.md: SETUP-6 and SETUP-7 cover points 1, 2, 3 and 5; the new intent line CHECK-1.f 'A result that a repository somebody else wrote carries along never hides a failure my check would report.'; choices 1-8 as recommended (tracked hashes.json neither read nor saved with one warning and the git rm --cached fix; confinement root = git top folder else --root, caches strict, absolute links refused with an 'absolute link' message; confine specs_dir, source_dirs and schema_dir with a budgeted confined walk; a changelog blob over 8 MiB skipped with a warning; import --from-dir confined to the named directory, 8 MiB; --fix --backup keeps its layout, written confined; the wider write scan; the two low items included); relayed by t4c over hc to t5r, to be recorded when change 5 starts after change 4); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree as 0xLeif

## What was verified

- [On base 1b3974e9 with only the new tests added (dfdaf4f6): 22 of 24 new integration tests and both new hash_cache unit tests fail (also REQ-cmd-check-020/021, REQ-hash-cache-009/010, REQ-cmd-import-004, REQ-cmd-init-registry-004). change answer and change approve wrote change.md through a committed link out and into a pre-commit hook; change answer created a dangling artifact link's outside target; change new built its workspace in a linked .specsync/changes; check --fix --backup copied into a linked backup-fix and over a linked backup file; init-registry created its registry in a linked .specsync; hooks install appended into a Git hook through CLAUDE.md; a subproject's links that stay in the repository were refused, and an absolute link was not refused as one; a tracked hashes.json was used by check and rewritten by rehash, and a forged pass in it replayed; a configured directory outside the project was not a config error, and a specs link out and a 257-level tree did not stop discovery; changelog gave no skip for an over-bound blob; import --from-dir imported through links out; prune kept keys outside the project; the write scan found 117 direct writes and the read scan 27 direct reads outside the helpers. The two controls (a subproject link out still refused; a backup with no link) pass.](evidence/2026-11-19-on-base-1b3974e9-with-only-the-new-tests-added-dfdaf4f6-22-of.md)
- [With the fix at e60fbf8e, all 24 new integration tests, both new hash_cache unit tests and the 7 new util unit tests pass (also REQ-cmd-check-020/021, REQ-hash-cache-009/010, REQ-cmd-import-004, REQ-cmd-init-registry-004, REQ-git-utils-012/013, REQ-agents-013 and the write-scan requirements), the write and read scans find no direct project file write or read outside the helpers, and the full verify lane passes: unit 3116 passed, 0 failed, 2 ignored; integration 689 passed, 0 failed; fmt, clippy, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts.](evidence/2026-11-19-with-the-fix-at-e60fbf8e-all-24-new-integration-tests-both-new.md)
- [On Linux at e60fbf8e (change 5), a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3120 passed, 0 failed, 2 ignored and the integration suite at 692 passed, 0 failed.](evidence/2026-11-19-on-linux-at-e60fbf8e-change-5-a-clean-copy-in-a-local-rust-1.md)

## Open threads

## Timeline

- 2026-11-19T20:02:24Z · start · intent → intent · Writes, reads and caches a repository can steer stay inside its Git work tree, never reach the Git directory, and never hide a failure (F2-F9, R1, R2, G1 and point 6 of the independent review, private, with GHSA-7q5v-8jcc-rx54) ^t-6c5c16
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - by: claude-code (hc t5r)
- 2026-11-19T20:06:42Z · gate-opened · intent → intent · Intent approval of proposed criterion CHECK-1.f ^t-2e0d4a
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - holds: intent
  - drives: none
  - by: claude-code (hc t5r)
- 2026-11-19T20:06:42Z · gate-opened · intent → intent · Scope approval of point 6 for keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree: refuse paths that resolve into the Git directory ^t-1a3a1e
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - holds: plan
  - drives: none
  - by: claude-code (hc t5r)
- 2026-11-19T20:06:42Z · gate-opened · intent → intent · Scope approval of keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree ^t-475e67
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-19T20:06:56Z · gate-closed · intent → intent · Intent approval of proposed criterion CHECK-1.f ^t-a832fd
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 16:00Z, after t4c summed up security/change5-scope.md: SETUP-6 and SETUP-7 cover points 1, 2, 3 and 5; the new intent line CHECK-1.f 'A result that a repository somebody else wrote carries along never hides a failure my check would report.'; choices 1-8 as recommended (tracked hashes.json neither read nor saved with one warning and the git rm --cached fix; confinement root = git top folder else --root, caches strict, absolute links refused with an 'absolute link' message; confine specs_dir, source_dirs and schema_dir with a budgeted confined walk; a changelog blob over 8 MiB skipped with a warning; import --from-dir confined to the named directory, 8 MiB; --fix --backup keeps its layout, written confined; the wider write scan; the two low items included); relayed by t4c over hc to t5r, to be recorded when change 5 starts after change 4
    - recorded_by: claude-code-t5r
  - by: claude-code (hc t5r)
- 2026-11-19T20:06:56Z · gate-closed · intent → intent · Scope approval of point 6 for keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree: refuse paths that resolve into the Git directory ^t-2efdff
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - relayed_by:
    - decided_by: 0xLeif
    - words: All as recommended
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 19:43Z, after t4c gave him one table of recommendations whose second row was point 6 of change 5 as proposed: the project read and write helpers refuse a path that resolves into the Git directory, directly or through links (canonical target under git rev-parse --absolute-git-dir, or any .git component, any case, trailing dots and spaces), with the message '<path> leads into the Git directory', and SpecSync's own .git writes keep their own paths; relayed by t4c over hc to t5r as part of change 5's scope
    - recorded_by: claude-code-t5r
  - by: claude-code (hc t5r)
- 2026-11-19T20:07:09Z · gate-closed · intent → intent · Scope approval of keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree ^t-7c94d4
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 16:00Z, after t4c summed up security/change5-scope.md: SETUP-6 and SETUP-7 cover points 1, 2, 3 and 5; the new intent line CHECK-1.f 'A result that a repository somebody else wrote carries along never hides a failure my check would report.'; choices 1-8 as recommended (tracked hashes.json neither read nor saved with one warning and the git rm --cached fix; confinement root = git top folder else --root, caches strict, absolute links refused with an 'absolute link' message; confine specs_dir, source_dirs and schema_dir with a budgeted confined walk; a changelog blob over 8 MiB skipped with a warning; import --from-dir confined to the named directory, 8 MiB; --fix --backup keeps its layout, written confined; the wider write scan; the two low items included); relayed by t4c over hc to t5r, to be recorded when change 5 starts after change 4
    - recorded_by: claude-code-t5r
    - native: specsync change approve keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T20:07:53Z · advance · intent → design ·  ^t-dc413a
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - by: claude-code (hc t5r)
- 2026-11-19T20:07:57Z · skip · design → plan · No visuals or interactions ^t-d7a4e7
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - by: claude-code (hc t5r)
- 2026-11-19T20:07:58Z · advance · plan → build · native:approved ^t-6dc108
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - gate: native:approved
  - revision: 1b3974e9a025f37ce712d4ad627e99f780d6f207
  - by: claude-code (hc t5r)
- 2026-11-19T20:34:04Z · advance · build → test · native:verifying ^t-70a4a2
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - gate: native:verifying
  - revision: e60fbf8e02faa643254590c1b4b5f5a659aa54d9
  - by: claude-code (hc t5r)
- 2026-11-19T20:34:07Z · advance · test → accept · native:verifying ^t-e497ec
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - gate: native:verifying
  - revision: e60fbf8e02faa643254590c1b4b5f5a659aa54d9
  - by: claude-code (hc t5r)
- 2026-11-19T20:34:07Z · gate-opened · accept → accept · Implementation review of keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree ^t-1d5e34
  - native_change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
  - revision: e60fbf8e02faa643254590c1b4b5f5a659aa54d9
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
