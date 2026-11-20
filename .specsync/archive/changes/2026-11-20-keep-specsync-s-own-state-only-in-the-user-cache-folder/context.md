---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: context
---

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
