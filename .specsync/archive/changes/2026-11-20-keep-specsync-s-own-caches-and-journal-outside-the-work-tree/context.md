---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: context
---

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
