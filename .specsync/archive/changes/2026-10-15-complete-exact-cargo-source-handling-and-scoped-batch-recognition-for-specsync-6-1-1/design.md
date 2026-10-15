---
change: complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1
artifact: design
---

# Design

Use one leaf-filename rule for generated scoped module paths and batch existence checks. Source detection must accept confined regular exact-file entries without scanning their parent, preserve explicit mappings, configured extension/test/exclusion policy, deduplicate overlapping entries and require exactly one candidate for fallback. Reuse shared detection where possible instead of divergent command copies.

Represent watch selection with path kind and registration mode: recursive directory or non-recursive exact regular file. Reject symlinks, escapes and unsupported paths; distinguish missing configuration from invalid configuration in text/JSON diagnostics. Preserve directory behavior, event filtering and debounce. Require a functioning watched-spec event control before reporting end-to-end source-edit reruns. Correct only the stale current release row while retaining historical immutable-tag caveats.

This is a bounded bug correction to existing commands. No new manifest discovery or ownership design is introduced.

## Exact-file lifecycle monitoring

The cached notify7 Linux backend removes direct file watches on DELETE_SELF. Keep the exact file NonRecursive registration and observe only its confined parent NonRecursive for replacement/recreate lifecycle events. Filter event paths to selected exact files or configured recursive directories; unrelated siblings do not trigger validation. Re-arm only after a new confined regular-file inspection. This is not a recursive project-root watcher or source selection expansion. Add normal-edit, atomic-save, later-edit and unlink/recreate controls with bounded owned subprocess cleanup.
