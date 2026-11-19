---
kind: session
id: 2026-11-19-bounded-project-reads
aliases: [bounded-project-reads]
objective: No SpecSync read of a project file follows a link out of the project, waits on a non-regular file, or reads past a size bound (private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules:
  - util
  - change
  - cmd_change
  - config
  - hash_cache
  - exports
  - validator
  - hooks
  - agents
  - registry
  - generator
  - cmd_migrate
  - cmd_check
  - commands
  - schema
  - deps
stage: accept
stage_status: active
iteration: 1
callers: []
native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
pr: ""
criteria: [SETUP-7]
requirements: []
cases: []
design: []
files: []
observations:
  - 2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17
  - 2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3
  - 2026-11-19-on-linux-at-3daf6ba0-a-clean-copy-in-a-local-rust-1-89
bugs: [2026-11-19-specsync-reads-project-files-through-links-waits-on-fifos-and]
blockers: []
waiting_on_human:
  - gate: Implementation review of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
    holds: accept
    drives: review
    opened: 2026-11-19T15:15:03Z
run_consents: []
next: Commit, chough change check, push to the private fork, Linux container run, then ask Leif through t4c for the implementation review
---
# No SpecSync read of a project file follows a link out of the project, waits on a non-regular file, or reads past a size bound (private, with GHSA-7q5v-8jcc-rx54)

## Goal

No SpecSync read of a project file follows a link out of the project, waits on a non-regular file, or reads past a size bound (private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

- Change 3 built: util's bounded project read, about 200 read sites moved onto it in 31 modules, a source scan guard, 31 specs, lesson, CHANGELOG and cli.md; full verify lane green

## Decisions and why

- 0xLeif decided "Scope approval of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound": "Approve" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:21Z, after t4c summed up the written change 3 scope in plain words (the confined, non-blocking, type-checked, bounded read helper; links inside followed; link out, dangling, FIFO, device, socket or over-bound refused naming path and reason, handled like that read's other errors; caches strict; bounds 4, 8 and 64 MiB; about 140 read sites; command-line paths, .git and Git objects out of scope; failing tests first, never /dev/zero); relayed by t4c over hc to t5r as his scope approval on exactly that text); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound as 0xLeif
- 0xLeif decided "Intent approval of proposed criterion SETUP-7": "OK as written" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:29Z, asked whether SETUP-7 'A command I run in a repository somebody else wrote never reads outside it through a link it found there, and no file there can make it hang or run out of memory.' is OK as written; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated
- agent choice (claude-code-t5r): Cargo config files above the project and in CARGO_HOME, and --template directories, keep a plain read because they are the person's own files or paths they name, which the approved scope leaves out
- agent choice (claude-code-t5r): merge --install refuses a linked .gitattributes before its read too, because its write already refused any link and the message should not depend on which step meets it

## What was verified

- [On base 3a02a5d5 with only the new tests added: 11 of 17 project-read integration tests fail. A FIFO at .specsync/sdd.json, the transaction journal, .specsync/config.toml, .specsyncignore, the MCP lock or CLAUDE.md made change status, change audit, change new, check, mcp diff and hooks status wait until the 20 s timeout killed them; sdd.json linked outside the project and a 4 MiB+1 sdd.json were read; a 64 MiB+1 hashes.json was read; and the source scan found 187 direct file read(s) bypassing a bounded read.](evidence/2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17.md)
- [With the fix, all 17 project-read integration tests and the 3 new util unit tests pass, the source scan finds no direct project file read left, and the full verify lane passes: unit 3098 passed, 0 failed, 2 ignored; integration 665 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts.](evidence/2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3.md)
- [On Linux at 3daf6ba0, a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3102 passed, 0 failed, 2 ignored and the integration suite at 668 passed, 0 failed.](evidence/2026-11-19-on-linux-at-3daf6ba0-a-clean-copy-in-a-local-rust-1-89.md)

## Open threads

## Timeline

- 2026-11-19T14:23:07Z · start · intent → intent · No SpecSync read of a project file follows a link out of the project, waits on a non-regular file, or reads past a size bound (private, with GHSA-7q5v-8jcc-rx54) ^t-413c1e
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - by: claude-code (hc t5r)
- 2026-11-19T14:27:11Z · gate-opened · intent → intent · Intent approval of proposed criterion SETUP-7 ^t-28643f
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - holds: intent
  - drives: none
  - by: claude-code (hc t5r)
- 2026-11-19T14:27:11Z · gate-opened · intent → intent · Scope approval of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound ^t-f2902c
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-19T14:27:11Z · gate-closed · intent → intent · Scope approval of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound ^t-b6f9c8
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:21Z, after t4c summed up the written change 3 scope in plain words (the confined, non-blocking, type-checked, bounded read helper; links inside followed; link out, dangling, FIFO, device, socket or over-bound refused naming path and reason, handled like that read's other errors; caches strict; bounds 4, 8 and 64 MiB; about 140 read sites; command-line paths, .git and Git objects out of scope; failing tests first, never /dev/zero); relayed by t4c over hc to t5r as his scope approval on exactly that text
    - recorded_by: claude-code-t5r
    - native: specsync change approve never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T14:30:21Z · gate-closed · intent → intent · Intent approval of proposed criterion SETUP-7 ^t-4d2573
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - relayed_by:
    - decided_by: 0xLeif
    - words: OK as written
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:29Z, asked whether SETUP-7 'A command I run in a repository somebody else wrote never reads outside it through a link it found there, and no file there can make it hang or run out of memory.' is OK as written; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
  - by: claude-code (hc t5r)
- 2026-11-19T14:30:21Z · advance · intent → design ·  ^t-e8b8c8
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - by: claude-code (hc t5r)
- 2026-11-19T14:30:21Z · skip · design → plan · Library and CLI change with no visuals or interactions ^t-ccbc42
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - by: claude-code (hc t5r)
- 2026-11-19T14:30:23Z · advance · plan → build · native:approved ^t-9e7c79
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - gate: native:approved
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - by: claude-code (hc t5r)
- 2026-11-19T15:15:00Z · advance · build → test · native:verifying ^t-51a0ca
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - gate: native:verifying
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - by: claude-code (hc t5r)
- 2026-11-19T15:15:03Z · advance · test → accept · native:verifying ^t-080427
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - gate: native:verifying
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - by: claude-code (hc t5r)
- 2026-11-19T15:15:03Z · gate-opened · accept → accept · Implementation review of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound ^t-234211
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
