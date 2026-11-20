---
kind: session
id: 2026-11-19-hash-cache-symlink-write
aliases: [hash-cache-symlink-write]
objective: specsync check and rehash never read or write the hash cache through a symbolic link, so a cloned repository cannot make them overwrite a file outside it (GHSA-7q5v-8jcc-rx54)
intent_scope: CHECK
modules: [hash_cache]
stage: done
stage_status: archived
iteration: 1
native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
criteria: [CHECK-1.e]
files: [src/hash_cache.rs, src/commands/check.rs, tests/integration/check.rs]
observations:
  - 2026-11-19-on-base-3bebe8b0-with-only-the-new-tests-added-5-of-6-new-hash
  - 2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both
bugs: [2026-11-19-check-and-rehash-write-the-hash-cache-through-a-symbolic-link]
title: Hash cache never read or written through a link
summary:
  - specsync check and rehash wrote .specsync/hashes.json with a write that follows a symbolic link, so a repository that committed that path as a link could make a check in a clone overwrite a file outside it (GHSA-7q5v-8jcc-rx54, 3.3.0 to 6.3.0).
  - The cache is now read and written through a no-follow handle on .specsync and saved by rename; any link is refused. check warns once and keeps its verdict and exit code, rehash exits 1.
  - "Verified: 5 unit and 2 integration tests failed first and pass; full macOS verify lane green; Linux arm64 container green. Next: the project file change on the same private PR, then the advisory."
closed: 2026-11-19
outcome: done
---
# specsync check and rehash never read or write the hash cache through a symbolic link, so a cloned repository cannot make them overwrite a file outside it (GHSA-7q5v-8jcc-rx54)

## Goal

specsync check and rehash never read or write the hash cache through a symbolic link, so a cloned repository cannot make them overwrite a file outside it (GHSA-7q5v-8jcc-rx54)

## Where it stands

- Failing tests first on 3bebe8b0: 5 unit and 2 integration tests red, recorded as evidence and as a bug
- HashCache load, save and refusal on a no-follow .specsync directory handle; check warns once before discovery and skips the cache; specs, lessons, CHANGELOG and docs updated

## Decisions and why

- 0xLeif decided "Intent approval of proposed criterion CHECK-1.e": "OK as written" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 04:25Z, asked whether CHECK-1.e 'Running the check on a repository somebody else wrote never changes a file outside that repository.' is OK as written; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated
- 0xLeif decided "Scope approval of never-read-or-write-the-hash-cache-through-a-symbolic-link": "Approve" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 04:25Z. t4c gave him the scope in plain words (load reads a linked or non-regular cache as empty; save refuses when .specsync or .specsync/hashes.json is a symlink or not a regular file, else writes a temp file in .specsync and renames it; check skips the cache with one stderr warning and keeps its verdict and exit code; rehash exits 1; the listed paths) and asked him to approve; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve never-read-or-write-the-hash-cache-through-a-symbolic-link as 0xLeif
- agent choice (claude-code-t5r): read and write through a cap-std directory handle opened with open_dir_nofollow, not path checks plus tempfile, because a path check followed by a path write follows a component swapped in between
- agent choice (claude-code-t5r): print the check warning before discovery because source discovery already ends the run on a linked .specsync, so a warning printed at cache load never appeared for that case and the approved REQ-cmd-check-018 criterion requires it
- rejected: replacing a linked hashes.json by rename because it hides a planted link and changes a tracked file without saying so
- 0xLeif decided "Implementation review of never-read-or-write-the-hash-cache-through-a-symbolic-link": "Approve" (Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:04Z, after t4c walked him through the change in plain words from its own read of the code at a33dcc6f, with its side effects and limits; relayed by t4c over hc to t5r as his implementation approval); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change review never-read-or-write-the-hash-cache-through-a-symbolic-link as 0xLeif with verdict pass

## What was verified

- On base 3bebe8b0 with only the new tests added: 5 of 6 new hash_cache unit tests and both new integration tests fail. check and rehash overwrite the outside file a linked .specsync/hashes.json names with the cache JSON, create a dangling link's target, write hashes.json into a linked .specsync, load a cache through either link, and write in place through a hard link. ^2026-11-19-on-base-3bebe8b0-with-only-the-new-tests-added-5-of-6-new-hash
- With the fix, the 6 new hash_cache unit tests and both integration tests pass: check (plain, --force, --strict, JSON, --spec auth JSON) and rehash leave a linked cache target byte for byte unchanged, create nothing through a dangling link or a linked .specsync, and check warns once with unchanged stdout and exit code. ^2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both

## Bugs

- [check and rehash write the hash cache through a symbolic link](../../bugs/2026-11-19-check-and-rehash-write-the-hash-cache-through-a-symbolic-link.md): verified

## Open threads

## Timeline

- 2026-11-19T04:23:06Z · start · intent → intent · specsync check and rehash never read or write the hash cache through a symbolic link, so a cloned repository cannot make them overwrite a file outside it (GHSA-7q5v-8jcc-rx54) ^t-a5b2c1
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t5r)
- 2026-11-19T04:24:31Z · gate-opened · intent → intent · Intent approval of proposed criterion CHECK-1.e ^t-7c90f2
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: intent
  - drives: none
  - by: claude-code (hc t5r)
- 2026-11-19T04:24:31Z · gate-opened · intent → intent · Scope approval of never-read-or-write-the-hash-cache-through-a-symbolic-link ^t-4e9544
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-19T04:26:02Z · gate-closed · intent → intent · Intent approval of proposed criterion CHECK-1.e ^t-61c292
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: OK as written
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 04:25Z, asked whether CHECK-1.e 'Running the check on a repository somebody else wrote never changes a file outside that repository.' is OK as written; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
  - by: claude-code (hc t5r)
- 2026-11-19T04:28:49Z · gate-closed · intent → intent · Scope approval of never-read-or-write-the-hash-cache-through-a-symbolic-link ^t-7ad964
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 04:25Z. t4c gave him the scope in plain words (load reads a linked or non-regular cache as empty; save refuses when .specsync or .specsync/hashes.json is a symlink or not a regular file, else writes a temp file in .specsync and renames it; check skips the cache with one stderr warning and keeps its verdict and exit code; rehash exits 1; the listed paths) and asked him to approve; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve never-read-or-write-the-hash-cache-through-a-symbolic-link
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T04:28:54Z · advance · intent → design ·  ^t-b7c4ec
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t5r)
- 2026-11-19T04:28:54Z · skip · design → plan · CLI and library change with no visuals or interactions ^t-7cd0e6
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t5r)
- 2026-11-19T04:28:54Z · advance · plan → build · native:approved ^t-3f64e7
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - gate: native:approved
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t5r)
- 2026-11-19T04:48:22Z · advance · build → test · native:verifying ^t-ba7d57
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - gate: native:verifying
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t5r)
- 2026-11-19T04:48:26Z · advance · test → accept · native:verifying ^t-b61591
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - gate: native:verifying
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - by: claude-code (hc t5r)
- 2026-11-19T04:48:26Z · gate-opened · accept → accept · Implementation review of never-read-or-write-the-hash-cache-through-a-symbolic-link ^t-85be86
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 3bebe8b03094b2079630801584b56f94a1e460c5
  - holds: accept
  - drives: review
  - by: claude-code (hc t5r)
- 2026-11-19T14:06:30Z · gate-closed · accept → accept · Implementation review of never-read-or-write-the-hash-cache-through-a-symbolic-link ^t-724cc4
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - revision: 7662f3ac815c54bfcee8a2ec45e9960e78623281
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve
    - where: Claude Code chat answer to his planning session (hc t4c) on 2026-11-19 at about 14:04Z, after t4c walked him through the change in plain words from its own read of the code at a33dcc6f, with its side effects and limits; relayed by t4c over hc to t5r as his implementation approval
    - recorded_by: claude-code-t5r
    - native: specsync change review never-read-or-write-the-hash-cache-through-a-symbolic-link
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-19T14:06:33Z · advance · accept → done · native:archived ^t-ad65bd
  - native_change: never-read-or-write-the-hash-cache-through-a-symbolic-link
  - gate: native:archived
  - revision: 7662f3ac815c54bfcee8a2ec45e9960e78623281
  - by: claude-code (hc t5r)

Closed 2026-11-19: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-19-on-base-3bebe8b0-with-only-the-new-tests-added-5-of-6-new-hash
    claim: "On base 3bebe8b0 with only the new tests added: 5 of 6 new hash_cache unit tests and both new integration tests fail. check and rehash overwrite the outside file a linked .specsync/hashes.json names with the cache JSON, create a dangling link's target, write hashes.json into a linked .specsync, load a cache through either link, and write in place through a hard link."
    result: failed
    revision: 3bebe8b03094b2079630801584b56f94a1e460c5
    environment: local-macos-arm64
    content: 40e1ce5714a4a00d48453d0d4d965a07f38808ab90c5364b5377f20e02e41ab0
    observed: 2026-11-19T04:31:02Z
    actor: claude-code (hc t5r)
  - id: 2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both
    claim: "With the fix, the 6 new hash_cache unit tests and both integration tests pass: check (plain, --force, --strict, JSON, --spec auth JSON) and rehash leave a linked cache target byte for byte unchanged, create nothing through a dangling link or a linked .specsync, and check warns once with unchanged stdout and exit code."
    result: passed
    revision: 3bebe8b03094b2079630801584b56f94a1e460c5
    environment: local-macos-arm64
    content: cba0c29436883f48d0b97280911b27f4a76fe37572df6382a92a6be54fea1345
    observed: 2026-11-19T04:48:07Z
    actor: claude-code (hc t5r)
```
