---
change: the-batched-object-reader-accepts-sha-256-object-ids
artifact: context
---

# Context

#985, a Codex P2 on #979 deferred by Leif's call (P2 policy: issue, reply, resolve).
`batchable_revision` (`src/git_utils.rs`) accepted `HEAD` or exactly 40 hex characters. In a
repository made with `git init --object-format=sha256` commits have 64-character ids, so every
`show <commit>:<path>` and `ls-tree ... <commit> -- <path>` read at a full commit id fell back to
its own `git show`/`git ls-tree`, while #979's lifecycle index (which accepts 40 or 64) skips the
prefetch workers. Cold archived status there spawned Git processes per archive instead of using
the one batch reader REQ-git-utils-010 requires. Verdicts were unchanged; only speed.

The fix accepts 64 hex characters as well. `cat-file --batch` handles both object formats, the
header parser is length-agnostic, and the tree walk already takes the hash length from the root
tree id. Reads through `HEAD` already worked, because the batch resolves HEAD itself.

Chough: spec-sync's `.chough/` is still the 0.5.x layout and the installed Chough 0.6.0 refuses
it; migrating it is its own PR and needs Leif's consent (raised with t370). This change runs the
SpecSync lifecycle only, as #1059 and #1062 did.
