# Lesson bundle: keep-specsync-s-own-state-only-in-the-user-cache-folder

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep SpecSync's own state only in the user cache folder
- **Kind**: BugFix
- **Specs**: util, change, hooks, git_utils, cmd_check
- **Paths**: src/util.rs, src/util/local_state.rs, src/git_utils.rs, src/change/sdd_version.rs, src/change_tests/journal_replay_targets.rs, src/hooks.rs, tests/integration
- **Acceptance**: SpecSync keeps its hash cache, transaction journal and remote spec cache only in specsync/projects/<key> in the user's cache folder (XDG_CACHE_HOME, ~/Library/Caches on macOS, ~/.cache elsewhere), keyed by the canonical project root, never in a Git directory: a forged cache, journal or remote spec entry an archive ships under its .git (or a .git file's gitdir) is never read. A cache folder that lies inside the canonical project root or inside the confinement's top folder is refused with the reason, so check warns once and validates every spec. tracked_by_git passes --literal-pathspecs and clears GIT_DIR, GIT_WORK_TREE, GIT_INDEX_FILE and the pathspec variables, so a project folder named * or notes[x] never widens the confinement. A journal entry is replayed for a registry-mapped spec, its requirements.md or history/ only when the mapped path ends in .spec.md. A hooks folder outside the work tree and the Git directory is made only when git config --show-scope says core.hooksPath is global or system; otherwise nothing missing is made there. The scans also report open_ambient_dir, File::open_ambient, .open( and std::fs aliases. The CHANGELOG, the CLI page, the design notes and the advisory say where the state lives and that an archive with .git cannot carry it; F5 says a FIFO blocks; N3 and R5b are documented. The new tests fail on e3d508ac with only the tests added and pass with the fix

## Evidence

- Verification commit: `7943890ab12ee5c8dcbf394a4d3683ac2d1f3872`
- Base commit: `e3d508ac254dd94817409b30fe180423de027a87`
- Verified by: `specsync check --spec change --spec cmd_check --spec git_utils --spec hooks --spec util`

## From the change's context.md

# Context

sec-review2 re-checked change 6 at `e3d508ac` (`security/review2/recheck-change6.md`, private) and put its confidence at 55%. It found:

- **R1, R2, R3.** The new state folder, `<git dir>/specsync/<key>`, can come with a tarball or ZIP that includes `.git`, or with a `.git` file whose `gitdir:` points inside the archive. A forged cache then passes a failing spec. A forged journal replays into a file the shipped registry names. A forged remote spec entry is used if it falls within the TTL. The key for a project at the top of its repository is a constant.
- **R4.** `tracked_by_git` passes the project folder to Git as a glob, so a ZIP whose folder is named `*` widens the confinement to the enclosing work tree again, which reopens F7.
- **R5a.** A hooks path outside the work tree and the Git directory can come from an archive's own `.git/config`. SpecSync then creates folders there.
- **N2.** A user cache folder inside the project, as in CI that sets `HOME` to the checkout, lets an archive with no `.git` ship the state.
- **R6.** Scan patterns are still missing: `open_ambient_dir(`, cap-std `Dir::open`, `File::open_ambient`, directory listings, and `std::fs` aliases.
- **Documentation:** F5 should say a FIFO blocks. The "where a repository cannot put a file" claims hold only for clones.

Human decision, relayed by the planning session (hc t4c): Leif answered "A" (2026-11-20, about 03:27Z) to:

> A (recommended): one more small change, about half a day. SpecSync keeps its cache only in your user cache folder, never in .git. That's simpler, one rule fewer, and closes the archive case and two related ones. It also fixes the one-line bug and the edge cases, then the reviewer does a quick re-check of just those.

t4c's brief lists items 1 to 5 and the text-and-tests items (`security/change7-scope.md`). R5b is out of scope and documented.

Changes 5 and 6 stay open. This change depends on change 6.

## From the change's design.md

# Design

- **`local_state::location`** always gives `<user cache>/specsync/projects/<sha256("project\0" + canonical root)[..16]>`.
  - `util::git_state_location` is removed.
  - The cache base is canonicalized, or its nearest existing ancestor is. A base inside the canonical root, or inside `util::confinement_top` (the folder the project file functions confine to), is refused with `RefusedLocalState` and the reason.
  - Old `.git/specsync/` state is never read or removed.
- **`git_utils::tracked_by_git`** runs `git --literal-pathspecs ls-files --error-unmatch -- <path>` and `git cat-file -e HEAD:./<path>`, with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_LITERAL_PATHSPECS`, `GIT_GLOB_PATHSPECS`, `GIT_NOGLOB_PATHSPECS` and `GIT_ICASE_PATHSPECS` removed.
- **`replayable_transaction_path`** accepts a registry entry's spec, its `requirements.md` and `history/` only when the mapped path ends in `.spec.md`.
- **`hooks::git_hooks_dir`.** When the hooks path lies outside both the work tree and the Git directory, `git config --show-scope --get core.hooksPath` is asked.
  - With `global` or `system`, missing folders may be made, as before.
  - Otherwise nothing missing is made. A missing folder is an error, and an existing one is used.
- **The scans** add `open_ambient_dir(`, `File::open_ambient(`, `.open(`, and `use std::fs as` / `std::fs::{self as` aliases. Each new finding is marked with its reason.

## From the change's testing.md

# Testing

Fixtures are temporary directories. "Outside" is a sibling directory there. Integration tests set `XDG_CACHE_HOME` to a folder in the system temporary directory, outside every fixture project.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-006 | `src/util/local_state.rs`: `local_state_never_lives_in_a_git_directory`, `local_state_lives_outside_the_work_tree`, `local_state_refuses_links_fifos_and_oversized_files`; `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_archive_ships_in_its_git_directory`, `check_refuses_a_user_cache_folder_inside_the_project` |
| REQ-util-008 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-git-utils-012 | `tests/integration/project_writes.rs`: `a_project_folder_named_like_a_glob_is_still_confined_to_itself`, `an_untracked_project_inside_another_work_tree_is_confined_to_itself`, `a_subproject_writes_through_a_link_that_stays_inside_its_repository` |
| REQ-change-162 | `src/change_tests/journal_replay_targets.rs`: `a_journal_under_the_git_directory_is_never_replayed`, `a_registry_entry_that_is_not_a_spec_never_opens_a_replay`, `a_registry_mapped_spec_is_recovered_and_its_neighbours_are_not`, `a_journal_in_the_work_tree_is_never_replayed` |
| REQ-hooks-009 | `tests/integration/project_writes.rs`: `hooks_install_makes_no_folder_for_a_local_hooks_path_outside_the_work_tree`, `hooks_install_still_makes_a_hooks_folder_from_the_global_config`, `hooks_install_never_writes_through_a_linked_folder_on_the_hooks_path` |
| REQ-cmd-check-021 | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_archive_ships_in_its_git_directory`, `check_refuses_a_user_cache_folder_inside_the_project`, `check_never_uses_a_hash_cache_git_tracks` |

Red first on `e3d508ac` with only change 7's records and tests added (`3fcb6de7`): all 3 new unit tests and 5 of the 7 new integration tests failed, and the read scan found 10 new unmarked opens. The 2 that passed are the global-config hooks control and the write scan, which has no new findings.

Then the full verify lane and the Linux container run.

## Where these lessons go

- `specs/util/util.3md`
- `specs/change/change.3md`
- `specs/hooks/hooks.3md`
- `specs/git_utils/git_utils.3md`
- `specs/cmd_check/cmd_check.3md`
