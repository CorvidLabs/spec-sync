# Lesson bundle: keep-specsync-s-own-caches-and-journal-outside-the-work-tree

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep SpecSync's own caches and journal outside the work tree
- **Kind**: BugFix
- **Specs**: util, hash_cache, change, cmd_check, rehash, cmd_resolve, hooks, validator
- **Paths**: src/util/local_state.rs, src/util.rs, src/hash_cache.rs, src/commands/check.rs, src/commands/rehash.rs, src/commands/resolve.rs, src/hooks.rs, src/change/sdd_version.rs, src/change/collect_requirement_ids.rs, src/validator/tests.rs, tests/integration
- **Acceptance**: SpecSync keeps its hash cache, transaction journal and remote spec cache outside the work tree: in <git rev-parse --absolute-git-dir>/specsync/<key>/ inside a Git work tree, keyed by the project's path in it, or in the user's cache folder (XDG_CACHE_HOME, ~/Library/Caches on macOS, ~/.cache elsewhere) keyed by the canonical project root when there is no work tree. A .specsync/hashes.json in the work tree, in any letter case and tracked or not, is never read or saved, so a forged pass in a clone, an unpacked tree or where Git cannot answer is never replayed. A .specsync/change-transaction.json in the work tree is never replayed: lifecycle commands refuse to start while it is there, with the path and how to clear it, without opening it. A .specsync-cache/remote-specs entry in the work tree is never used by resolve. The read and write scans cover exactly the files compiled into the binary (from Cargo's dep-info), skip only #[cfg(test)] items found by a lexer that ignores braces in strings, characters and comments, and also catch cap-std opens, renames and copies, tempfile_in and persist, hard and symbolic links, free functions imported from std::fs, and git arguments that write files; the hidden unbounded read in src/validator/tests.rs goes through the bounded read. The pre-commit hook is read, written, created, made executable and removed through a hooks folder opened component by component without following links, from the Git directory, the work tree top or the user's own folder, so a committed hooks folder link or a dangling pre-commit link is refused. The confinement widens to an enclosing work tree only when Git tracks the project root there. --template and --manifest stay as given; F5, F8, F9 and F10 of the second review are documented. The new tests fail on a2a50824 with only the tests added and pass with the fix

## Evidence

- Verification commit: `33f018109b44f5a8e8d37c46d8aebe9ad1fb76de`
- Base commit: `a2a508244b3be174a57f1b681eba5c62cfb76661`
- Verified by: `specsync check --spec change --spec cmd_check --spec cmd_resolve --spec hash_cache --spec hooks --spec rehash --spec util --spec validator`

## From the change's context.md

# Context

A second independent review of the private advisory branch at `a2a50824` (`security/review2/report.md`, private) found that a repository can still ship state SpecSync trusts, plus gaps in the guards:

- **F1:** `.specsync/hashes.json` is trusted whenever Git does not say it is tracked. That covers a case variant such as `.specsync/Hashes.json` on macOS's case-insensitive disk, an unpacked ZIP or tarball, and Git failing (for example "dubious ownership" in container CI). A plain `specsync check` then replays a forged pass.
- **F2:** the transaction journal has the same tracked-check gap. Its replay can rewrite files under `.specsync/`, including a forged untracked cache, and any file the committed registry names.
- **F3:** `resolve --verify` trusts a committed `.specsync-cache/remote-specs/*`.
- **F4:** the read and write scans skip every file named `tests.rs`, and `src/validator/tests.rs` holds production code with an unbounded read. Their brace counting runs past string literals that hold a brace, and the call list misses cap-std opens, renames and copies, `tempfile_in`, links, imported functions and Git arguments that write.
- **F6:** `hooks install --precommit` trusts whatever hooks folder Git names. With `core.hooksPath` inside the work tree, a committed parent link or a dangling `pre-commit` link sends the write, and the chmod to 0755, outside.
- **F7:** a project that is not its own repository, unpacked inside another work tree, is confined to the outer tree.
- **F5, F8, F9 and F10:** rated low or needing local access.

Human decision, relayed by the planning session (hc t4c): Leif answered "a, all yes.. im back" (2026-11-20, about 00:38Z) to option A of item 1 as t4c put it at about 21:09Z on 2026-11-19:

> one more change (change 6): move the hash cache, journal and remote cache out of the work tree; close the scan blind spots, the hooks-folder case and the enclosing-repo case; leave the paths you type yourself (--template) as they are, and document the low-rated items. Then this same reviewer re-checks its own findings [...] (fold [change 5's review] into change 6 if you pick A)

The same message said the state moves "into the Git directory, where a repo can't commit files, or into your user cache folder when there's no Git."

Change 5 (`keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree`) stays verifying, with its review gate open. Leif reviews both together.

## From the change's design.md

# Design

## Local state outside the work tree (F1, F2, F3)

New module `src/util/local_state.rs`, a child module of `util`.

**`project_state_dir(root)` says where the state lives:**
- **In a Git work tree:** `<absolute-git-dir>/specsync/<key>`. The key is the first 16 hex digits of SHA-256 over the project root's path in the work tree.
- **With no work tree:** `<user cache>/specsync/projects/<key>`. The key is the same digest over the canonical project root.
  - The user cache is `$XDG_CACHE_HOME` when it is set to an absolute path; otherwise `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows, and `~/.cache` elsewhere.
  - Unit tests use a folder in the system temporary directory, so they never touch the user's cache.

**`LocalState`** is a directory handle on that folder, opened without following a link at any name SpecSync creates (`specsync`, the key, `projects`).
- **Reads** are non-blocking, type-checked on the handle and bounded.
- **Writes** create a new temporary file and rename it over the name. A durable write also syncs the file and the folder.
- **A remove** unlinks the name.

**The three files there:**
- **The hash cache: `hashes.json`.** The work tree's `.specsync/hashes.json` is never looked at, so change 5's tracked check and its `git` processes go. `check` prints the real path in its "Cache:" line, and `rehash` prints where it wrote. With no state folder, `check` warns once and validates everything, and `rehash` exits 1.
- **The journal: `change-transaction.json`.** It is written, replayed and removed only there. A `.specsync/change-transaction.json` in the work tree, of any kind, makes the lifecycle lock refuse with its path and how to clear it, and it is never opened. Change 4's per-entry rule stays.
- **The remote spec cache: `remote-specs/<sha256 of repo and path>`.** The work tree's `.specsync-cache` is never looked at.

## Confinement (F7)

`work_tree_confinement` keeps the work tree's top folder only when the prefix is empty, or when `git ls-files --error-unmatch -- ':(top,literal)<prefix>'` exits 0. Otherwise the top is the project root. The Git directory is kept for the refusal either way. The local state key uses the real prefix.

## The hooks folder (F6)

`git_hooks_dir` returns a `HooksDir`: a handle reached component by component without following any link. It starts from one of these bases:
- the canonical Git directory, when the hooks path lies in it;
- the canonical work-tree top, when it lies there;
- otherwise the canonical parent of the user's own folder.

`pre-commit` is read, replaced, created, made executable and removed through that handle:
- **read:** no-follow, non-blocking, a type check on the handle, 8 MiB;
- **replace:** open without create, check the type, truncate, write;
- **create:** exclusive create;
- **make executable:** `fchmod` on the handle;
- **remove:** unlink.

A link on the way, or at `pre-commit`, is refused with the path named.

## The scans (F4)

- **Which files.** The read and write scans take the files to scan from Cargo's dep-info for the binary. They fail if it cannot be read or does not list `src/main.rs`.
- **`#[cfg(test)]` items** are found by a lexer that skips line and block comments, normal, raw and byte strings, and character literals.
- **New read calls:** a bare `read_to_string(`, `.open_with(`, `open_ambient_dir(`, `open_dir_nofollow(`, `.open_dir(`.
- **New write calls:** `hard_link(`, `symlink(`, `tempfile_in(`, `NamedTempFile::new_in(`, `tempdir_in(`, `persist_noclobber(`, `.open_with(`, `.rename(`, `.copy(`, `set_len(`.
- **Imports:** a `use std::fs::` import that names a free function is reported as a read or a write.
- **A third scan**, `no_spawned_git_command_writes_files`, reports a Git argument that writes files (`--output`, `worktree`, `checkout`, `restore`, `stash`, `clean`, `reset`, `archive`, `bundle`) unless it is marked `git writes no project file:` with the reason.
- **The hidden read** in `compute_coverage_checked` goes through `read_project_string` with the 8 MiB bound.
- **Wrong markers are corrected** in `hooks.rs` and `validate_loaded_change.rs`.

## From the change's testing.md

# Testing

Fixtures are temporary directories. Every link points inside the fixture's own temporary directory, and "outside" is a sibling directory there. Integration tests set `XDG_CACHE_HOME` to a folder in the fixture, so nothing is written to the user's cache.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-006 | `src/util/local_state.rs`: `local_state_lives_outside_the_work_tree`, `local_state_refuses_links_fifos_and_oversized_files` |
| REQ-util-007 | `tests/integration/project_writes.rs`: `an_untracked_project_inside_another_work_tree_is_confined_to_itself`, `a_subproject_writes_through_a_link_that_stays_inside_its_repository`, `a_subproject_reads_specs_through_a_link_that_stays_inside_its_repository`, `a_subproject_link_that_leaves_its_repository_is_still_refused` |
| REQ-util-008 | `tests/integration/project_reads.rs`: `the_scan_lexer_blanks_only_comments_and_literals`, `no_project_file_is_read_outside_the_bounded_read`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`, `no_spawned_git_command_writes_files_unmarked` |
| REQ-hash-cache-009 | `src/hash_cache.rs`: `a_forged_pass_an_unpacked_tree_ships_is_never_replayed`, `a_forged_pass_committed_in_another_letter_case_is_never_replayed`, `a_forged_pass_in_a_cache_git_tracks_is_never_replayed`, `a_link_at_the_work_tree_cache_path_is_never_followed`, `save_creates_nothing_through_a_dangling_link_or_a_linked_specsync`, `load_reads_no_cache_through_a_link` |
| REQ-cmd-check-021 | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_unpacked_tree_ships`, `check_never_uses_a_hash_cache_git_tracks`; `tests/integration/check.rs`: `check_never_writes_the_hash_cache_through_a_link`; `tests/integration/config.rs`: `check_creates_hash_cache` |
| REQ-rehash-003 | `tests/integration/check.rs`: `rehash_never_writes_the_hash_cache_through_a_link`; `tests/integration/repository_inputs.rs`: `rehash_leaves_a_hash_cache_git_tracks` |
| REQ-change-162 | `src/change_tests/journal_replay_targets.rs`: `a_journal_in_the_work_tree_is_never_replayed`, `a_running_transaction_keeps_its_journal_outside_the_work_tree`, `a_journal_git_tracks_is_never_replayed`, `a_journal_never_rewrites_or_deletes_git_files`; `tests/integration/repository_inputs.rs`: `change_new_never_replays_a_journal_an_unpacked_tree_ships`; `tests/integration/project_reads.rs`: `a_lifecycle_write_does_not_wait_on_or_follow_the_transaction_journal`, `change_new_never_replays_a_committed_journal_into_git` |
| REQ-cmd-resolve-002 | `src/commands/resolve.rs`: `spec_cache_never_returns_an_entry_the_work_tree_holds`, `spec_cache_never_writes_or_reads_through_a_link`, `spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory`, `test_spec_cache_roundtrip` |
| REQ-cmd-resolve-003 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |
| REQ-hooks-009 | `tests/integration/project_writes.rs`: `hooks_install_never_writes_through_a_linked_folder_on_the_hooks_path`, `hooks_install_never_creates_the_target_of_a_dangling_pre_commit_link`, `hooks_install_installs_into_a_hooks_folder_in_the_work_tree`; `src/hooks.rs`: `install_and_uninstall_precommit_refuse_symlinked_hook` |
| REQ-validator-052 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |

Red first on `a2a50824` with only the tests commit added (`b2c70e1f`): all 5 new unit tests and 8 of the 13 new integration tests failed. The read scan found 12 unmarked reads, `src/validator/tests.rs:3452` among them, the write scan 7 and the Git scan 5. The 5 that passed there are as they should be: the three monorepo controls, a real in-tree hooks folder, and the lexer test. The `src/util/local_state.rs` tests call the module this change adds, so they could not run on the base. What they cover is also driven by the CLI tests above.

Then the full verify lane and the Linux container run.

## Where these lessons go

- `specs/util/util.3md`
- `specs/hash_cache/hash_cache.3md`
- `specs/change/change.3md`
- `specs/cmd_check/cmd_check.3md`
- `specs/rehash/rehash.3md`
- `specs/cmd_resolve/cmd_resolve.3md`
- `specs/hooks/hooks.3md`
- `specs/validator/validator.3md`
