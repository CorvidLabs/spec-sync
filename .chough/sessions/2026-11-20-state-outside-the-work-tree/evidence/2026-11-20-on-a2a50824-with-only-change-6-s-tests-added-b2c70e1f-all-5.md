---
kind: evidence
id: 2026-11-20-on-a2a50824-with-only-change-6-s-tests-added-b2c70e1f-all-5
aliases: []
claim: "On a2a50824 with only change 6's tests added (b2c70e1f): all 5 new unit tests and 8 of 13 new integration tests fail (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-cmd-resolve-002). A forged pass in .specsync/hashes.json replays in an unpacked tree and, on this case-insensitive disk, when committed as .specsync/Hashes.json; check in an unpacked copy skips the spec on a shipped cache; a valid journal planted in the work tree is replayed by recover and by change new, with or without Git; a running transaction writes its journal into the work tree; a committed .specsync-cache/remote-specs entry is returned by the remote cache; hooks install --precommit with core.hooksPath in the work tree created outside/hooks through a linked .config and outside/created through a dangling pre-commit link; an untracked project in a home folder kept under Git appended to the outer notes.md. The read scan finds 12 unmarked reads (src/validator/tests.rs:3452 among them), the write scan 7, the Git scan 5 (git worktree add/remove/prune). The 5 that pass are controls: three monorepo subproject tests, a real in-tree hooks folder, and the lexer test."
result: failed
evidence_kind: product-test
revision: b2c70e1fe315e15d2305563920bb6cba9ecb4550
content: 3633f56f0f0e3efe9e65bd00972f0125d37a593ef594dc3205214c762b8c8013
environment: macOS arm64 (APFS, case-insensitive), rustc 1.89.0, isolated git config, through one-lane; tree of b2c70e1f (a2a50824 + the tests only)
fingerprint: sha256:85ef1dd11116389daeef88b5b24b2905406ba8a32bebabf4afa88ebc55d67c2a
external: ext-97165b
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements:
  - REQ-util-006
  - REQ-util-007
  - REQ-util-008
  - REQ-change-162
  - REQ-hooks-009
  - REQ-validator-052
contracts: []
limitations: []
observed: 2026-11-20T01:12:01Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On a2a50824 with only change 6's tests added (b2c70e1f): all 5 new unit tests and 8 of 13 new integration tests fail (also REQ-hash-cache-009, REQ-cmd-check-021, REQ-cmd-resolve-002). A forged pass in .specsync/hashes.json replays in an unpacked tree and, on this case-insensitive disk, when committed as .specsync/Hashes.json; check in an unpacked copy skips the spec on a shipped cache; a valid journal planted in the work tree is replayed by recover and by change new, with or without Git; a running transaction writes its journal into the work tree; a committed .specsync-cache/remote-specs entry is returned by the remote cache; hooks install --precommit with core.hooksPath in the work tree created outside/hooks through a linked .config and outside/created through a dangling pre-commit link; an untracked project in a home folder kept under Git appended to the outer notes.md. The read scan finds 12 unmarked reads (src/validator/tests.rs:3452 among them), the write scan 7, the Git scan 5 (git worktree add/remove/prune). The 5 that pass are controls: three monorepo subproject tests, a real in-tree hooks folder, and the lexer test.**

Recorded as failed in macOS arm64 (APFS, case-insensitive), rustc 1.89.0, isolated git config, through one-lane; tree of b2c70e1f (a2a50824 + the tests only) at revision b2c70e1fe315, observed 2026-11-20T01:12:01Z.

## Key failing lines

- hash_cache::tests::a_forged_pass_committed_in_another_letter_case_is_never_replayed: a cache committed as .specsync/Hashes.json replayed a forged pass | repository_inputs::check_never_uses_a_hash_cache_an_unpacked_tree_ships: Skipped 1 unchanged spec(s) | project_writes::hooks_install_never_creates_the_target_of_a_dangling_pre_commit_link: left ["created"] right [] | project_reads: 12 direct file read(s) ... src/validator/tests.rs:3452
