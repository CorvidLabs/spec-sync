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
stage: done
stage_status: archived
iteration: 1
native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
criteria: [SETUP-7]
observations:
  - 2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17
  - 2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3
  - 2026-11-19-on-linux-at-3daf6ba0-a-clean-copy-in-a-local-rust-1-89
bugs: [2026-11-19-specsync-reads-project-files-through-links-waits-on-fifos-and]
title: Project file reads are confined and bounded
summary:
  - SpecSync read about 200 project files with reads that follow links, block on FIFOs and have no bound, so a committed link from .specsync/sdd.json to /dev/zero exhausted memory in change status, which Chough runs on every next and status.
  - "Every in-scope read now goes through util's bounded project read: confined to the root, non-blocking, type-checked on the handle, at most the bound plus one byte (4, 8 and 64 MiB); a source scan test keeps it that way."
  - "Verified: 11 of 17 integration tests and the scan failed first and pass; full macOS verify lane green; Linux arm64 container green. Next: the advisory text, then publishing and release on Leif's yes."
closed: 2026-11-19
outcome: done
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
- 0xLeif decided "Implementation review of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound": "yes, change 3 too" (Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:36Z, after t4c walked him through the change in plain words from its own read of read_project_file at 2446aba8 (confined open, non-blocking, type check on the open handle, bound+1 read; the source scan test; about 200 sites; the out-of-scope reads marked in source; the red-first and green counts on macOS and Linux; the three side effects; x86 and CI Ubuntu not run) and asked whether its implementation review passes; relayed by t4c over hc to t5r as his implementation review); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change review never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound as 0xLeif with verdict pass

## What was verified

- On base 3a02a5d5 with only the new tests added: 11 of 17 project-read integration tests fail. A FIFO at .specsync/sdd.json, the transaction journal, .specsync/config.toml, .specsyncignore, the MCP lock or CLAUDE.md made change status, change audit, change new, check, mcp diff and hooks status wait until the 20 s timeout killed them; sdd.json linked outside the project and a 4 MiB+1 sdd.json were read; a 64 MiB+1 hashes.json was read; and the source scan found 187 direct file read(s) bypassing a bounded read. ^2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17
- With the fix, all 17 project-read integration tests and the 3 new util unit tests pass, the source scan finds no direct project file read left, and the full verify lane passes: unit 3098 passed, 0 failed, 2 ignored; integration 665 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts. ^2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3
- On Linux at 3daf6ba0, a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3102 passed, 0 failed, 2 ignored and the integration suite at 668 passed, 0 failed. ^2026-11-19-on-linux-at-3daf6ba0-a-clean-copy-in-a-local-rust-1-89

## Bugs

- [SpecSync reads project files through links, waits on FIFOs and reads without a bound](../../bugs/2026-11-19-specsync-reads-project-files-through-links-waits-on-fifos-and.md): verified

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
- 2026-11-19T15:48:12Z · gate-closed · accept → accept · Implementation review of never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound ^t-a5d273
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - revision: c2f5182999e5e86eb81f7bd260671b7e90022ffa
  - relayed_by:
    - decided_by: 0xLeif
    - words: yes, change 3 too
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-19 at about 15:36Z, after t4c walked him through the change in plain words from its own read of read_project_file at 2446aba8 (confined open, non-blocking, type check on the open handle, bound+1 read; the source scan test; about 200 sites; the out-of-scope reads marked in source; the red-first and green counts on macOS and Linux; the three side effects; x86 and CI Ubuntu not run) and asked whether its implementation review passes; relayed by t4c over hc to t5r as his implementation review
    - recorded_by: claude-code-t5r
    - native: specsync change review never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T15:48:21Z · advance · accept → done · native:archived ^t-67c87c
  - native_change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
  - gate: native:archived
  - revision: c2f5182999e5e86eb81f7bd260671b7e90022ffa
  - by: claude-code (hc t5r)

Closed 2026-11-19: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-19-on-base-3a02a5d5-with-only-the-new-tests-added-11-of-17
    claim: "On base 3a02a5d5 with only the new tests added: 11 of 17 project-read integration tests fail. A FIFO at .specsync/sdd.json, the transaction journal, .specsync/config.toml, .specsyncignore, the MCP lock or CLAUDE.md made change status, change audit, change new, check, mcp diff and hooks status wait until the 20 s timeout killed them; sdd.json linked outside the project and a 4 MiB+1 sdd.json were read; a 64 MiB+1 hashes.json was read; and the source scan found 187 direct file read(s) bypassing a bounded read."
    result: failed
    revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
    environment: local-macos-arm64
    content: ea398bda76bb12c41e999ca2b92fc9e65dece1725519845858be0c10dc8e14b4
    observed: 2026-11-19T14:30:41Z
    actor: claude-code (hc t5r)
  - id: 2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3
    claim: "With the fix, all 17 project-read integration tests and the 3 new util unit tests pass, the source scan finds no direct project file read left, and the full verify lane passes: unit 3098 passed, 0 failed, 2 ignored; integration 665 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts."
    result: passed
    revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
    environment: local-macos-arm64
    content: c61d228eac8868b38f9d758247a7fb5b936151d7e26b536900f8eb290e639d9b
    observed: 2026-11-19T15:13:01Z
    actor: claude-code (hc t5r)
  - id: 2026-11-19-on-linux-at-3daf6ba0-a-clean-copy-in-a-local-rust-1-89
    claim: "On Linux at 3daf6ba0, a clean copy in a local rust:1.89-bookworm container, run as a normal user with jq installed: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with the unit suite at 3102 passed, 0 failed, 2 ignored and the integration suite at 668 passed, 0 failed."
    result: passed
    revision: 3daf6ba003c3ba4e9aef64c4a9d5767fb5bb0b5b
    environment: local-linux-arm64-container
    content: 8919ddb2458082958a65bf2d700905bea3cf3ecc6af23487346d667aa41bd582
    observed: 2026-11-19T15:21:03Z
    actor: claude-code (hc t5r)
```
