---
id: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
state: implementing
type: bug_fix
base_commit: a2a508244b3be174a57f1b681eba5c62cfb76661
---

# Keep SpecSync's own caches and journal outside the work tree

## Intent

Keep SpecSync's own caches and journal outside the work tree

## Affected Canonical Specs

- `util`
- `hash_cache`
- `change`
- `cmd_check`
- `rehash`
- `cmd_resolve`
- `hooks`
- `validator`

## Acceptance Criteria

- SpecSync keeps its hash cache, transaction journal and remote spec cache outside the work tree: in <git rev-parse --absolute-git-dir>/specsync/<key>/ inside a Git work tree, keyed by the project's path in it, or in the user's cache folder (XDG_CACHE_HOME, ~/Library/Caches on macOS, ~/.cache elsewhere) keyed by the canonical project root when there is no work tree. A .specsync/hashes.json in the work tree, in any letter case and tracked or not, is never read or saved, so a forged pass in a clone, an unpacked tree or where Git cannot answer is never replayed. A .specsync/change-transaction.json in the work tree is never replayed: lifecycle commands refuse to start while it is there, with the path and how to clear it, without opening it. A .specsync-cache/remote-specs entry in the work tree is never used by resolve. The read and write scans cover exactly the files compiled into the binary (from Cargo's dep-info), skip only #[cfg(test)] items found by a lexer that ignores braces in strings, characters and comments, and also catch cap-std opens, renames and copies, tempfile_in and persist, hard and symbolic links, free functions imported from std::fs, and git arguments that write files; the hidden unbounded read in src/validator/tests.rs goes through the bounded read. The pre-commit hook is read, written, created, made executable and removed through a hooks folder opened component by component without following links, from the Git directory, the work tree top or the user's own folder, so a committed hooks folder link or a dangling pre-commit link is refused. The confinement widens to an enclosing work tree only when Git tracks the project root there. --template and --manifest stay as given; F5, F8, F9 and F10 of the second review are documented. The new tests fail on a2a50824 with only the tests added and pass with the fix

## No-spec Rationale

Not applicable
