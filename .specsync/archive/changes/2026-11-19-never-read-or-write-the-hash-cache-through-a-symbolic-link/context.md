---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: context
---

# Context

Reported privately as GHSA-7q5v-8jcc-rx54 and handled under `SECURITY.md`. `HashCache::save` wrote `.specsync/hashes.json` with `fs::write`, which follows a symbolic link. A repository can commit that path as a link, since Git stores links and a `.gitignore` entry does not stop a forced add. Anyone who then clones it and runs `specsync check` (which saves the cache after a run with no errors) or `specsync rehash` overwrites the file the link names, outside the repository, with the cache JSON. The repository chooses the JSON's keys: they are its spec and source paths. A dangling link creates the file. `.specsync` itself can be the link, which puts `hashes.json` into a directory outside the repository. `HashCache::load` read through the same links.

Human intent: CHECK-1.e, "Running the check on a repository somebody else wrote never changes a file outside that repository."

What a session picking this up needs to know:

- The work stays in the advisory's private fork until the advisory is published. Nothing about it goes to the public repository, an issue or a comment before then.
- The scope the human approved: load reads a linked or non-regular cache as empty; save refuses when `.specsync` or `.specsync/hashes.json` is a symbolic link or not a regular file, and otherwise writes a new file in `.specsync` and renames it; `check` skips the cache with one stderr warning and keeps its verdict and exit code; `rehash` exits 1.
- Ruled out: replacing a linked `hashes.json` with a regular file. A rename would be safe, but it hides a planted link and changes a tracked file without saying so.
- Not in this change: other writers under `.specsync`, and the files a spec's `files:` list makes the cache hash. A separate private sweep covers them.
