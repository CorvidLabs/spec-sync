---
id: keep-specsync-s-own-state-only-in-the-user-cache-folder
state: archived
type: bug_fix
base_commit: e3d508ac254dd94817409b30fe180423de027a87
---

# Keep SpecSync's own state only in the user cache folder

## Intent

Keep SpecSync's own state only in the user cache folder

## Affected Canonical Specs

- `util`
- `change`
- `hooks`
- `git_utils`
- `cmd_check`

## Acceptance Criteria

- SpecSync keeps its hash cache, transaction journal and remote spec cache only in specsync/projects/<key> in the user's cache folder (XDG_CACHE_HOME, ~/Library/Caches on macOS, ~/.cache elsewhere), keyed by the canonical project root, never in a Git directory: a forged cache, journal or remote spec entry an archive ships under its .git (or a .git file's gitdir) is never read. A cache folder that lies inside the canonical project root or inside the confinement's top folder is refused with the reason, so check warns once and validates every spec. tracked_by_git passes --literal-pathspecs and clears GIT_DIR, GIT_WORK_TREE, GIT_INDEX_FILE and the pathspec variables, so a project folder named * or notes[x] never widens the confinement. A journal entry is replayed for a registry-mapped spec, its requirements.md or history/ only when the mapped path ends in .spec.md. A hooks folder outside the work tree and the Git directory is made only when git config --show-scope says core.hooksPath is global or system; otherwise nothing missing is made there. The scans also report open_ambient_dir, File::open_ambient, .open( and std::fs aliases. The CHANGELOG, the CLI page, the design notes and the advisory say where the state lives and that an archive with .git cannot carry it; F5 says a FIFO blocks; N3 and R5b are documented. The new tests fail on e3d508ac with only the tests added and pass with the fix

## No-spec Rationale

Not applicable
