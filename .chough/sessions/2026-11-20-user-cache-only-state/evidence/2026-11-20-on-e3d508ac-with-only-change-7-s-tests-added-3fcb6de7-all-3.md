---
kind: evidence
id: 2026-11-20-on-e3d508ac-with-only-change-7-s-tests-added-3fcb6de7-all-3
aliases: []
claim: "On e3d508ac with only change 7's tests added (3fcb6de7): all 3 new unit tests and 5 of 7 new integration tests fail (also REQ-git-utils-012, REQ-cmd-check-021). The state folder of a Git work tree is .git/specsync/<constant key>; a copy of a project with its .git replays the cache it ships (Skipped 1 unchanged spec, Cache: unpacked/.git/specsync/...); a valid journal at .git/specsync/<key>/change-transaction.json is replayed; a registry entry env = .env lets a journal delete .env; with XDG_CACHE_HOME inside the project the cache there is replayed; a project folder named * inside a home work tree that tracks Downloads/readme.txt is widened and the outer notes.md gets the CLAUDE.md block; a local core.hooksPath ../outside/made/hooks makes outside/made. The read scan finds 10 new unmarked opens (open_ambient_dir and .open(). The 2 that pass are the global-config hooks control and the write scan."
result: failed
evidence_kind: product-test
revision: 3fcb6de7053366ab2b8d4a5246eaa9b247939e89
content: f20ad66eda1be07d9d2700ab0f9fca4ae14fb03c7158babb6fd27547e7a325c2
environment: macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tree of 3fcb6de7 (e3d508ac + change 7's records and tests only)
fingerprint: sha256:aa031ad590932c5c4641b641b0756a1c7a88afd0956c497984ea30a930f5281c
external: ext-6b063e
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: [REQ-util-006, REQ-util-008, REQ-change-162, REQ-hooks-009]
contracts: []
limitations: []
observed: 2026-11-20T03:37:11Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On e3d508ac with only change 7's tests added (3fcb6de7): all 3 new unit tests and 5 of 7 new integration tests fail (also REQ-git-utils-012, REQ-cmd-check-021). The state folder of a Git work tree is .git/specsync/<constant key>; a copy of a project with its .git replays the cache it ships (Skipped 1 unchanged spec, Cache: unpacked/.git/specsync/...); a valid journal at .git/specsync/<key>/change-transaction.json is replayed; a registry entry env = .env lets a journal delete .env; with XDG_CACHE_HOME inside the project the cache there is replayed; a project folder named * inside a home work tree that tracks Downloads/readme.txt is widened and the outer notes.md gets the CLAUDE.md block; a local core.hooksPath ../outside/made/hooks makes outside/made. The read scan finds 10 new unmarked opens (open_ambient_dir and .open(). The 2 that pass are the global-config hooks control and the write scan.**

Recorded as failed in macOS arm64 (APFS), rustc 1.89.0, isolated git config, through one-lane; tree of 3fcb6de7 (e3d508ac + change 7's records and tests only) at revision 3fcb6de70533, observed 2026-11-20T03:37:11Z.

## Key failing lines

- local_state_never_lives_in_a_git_directory: the state folder is inside the clone: .../repo/.git/specsync/f86e7b478718b160 | check_never_uses_a_hash_cache_an_archive_ships_in_its_git_directory: Skipped 1 unchanged spec(s) | a_registry_entry_that_is_not_a_spec_never_opens_a_replay: left None | hooks_install_makes_no_folder_for_a_local_hooks_path_outside_the_work_tree: left [made]
