---
change: supersede-reads-the-predecessor-digest-says-when-the-predecessor-is-on-another-branch-and-checks-the-base-entry-when
artifact: design
---

# Design

- **Optional digest.** `--digest` becomes optional. Without it, supersede reads the entry for
  `--path` from the predecessor's signed acceptance manifest (`resolved_acceptance_manifest`):
  exactly one entry, or a refusal naming none or ambiguous. The predecessor must be accepted or
  archived, as before.
- **Other branch.** When `find_change_dir` finds no active or archived change for the
  predecessor, the refusal says it is not in this branch's history and to merge or rebase onto the
  branch that has it. Other lookup errors (closed, closed late, ambiguous) pass through.
- **Base entry, when declared.** `succession_base_warning` compares the predecessor's entry at the
  successor's base commit (`acceptance_entry_digest_at_commit`, the same check coverage uses
  later) with the obligation. On a mismatch it names the commit that last changed the path before
  the base, any accepted or archived change whose manifest signed the file as it is at the base,
  and the two ways forward: supersede that change, or `change reconcile-history`. It is a
  warning on stderr, printed after the obligation is recorded, because reconcile-history needs the
  obligation for exactly this case; refusing would block it.

Rejected: refusing a moved base at supersede time (it blocks reconcile-history).
