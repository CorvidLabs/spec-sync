---
change: CHG-0163-a-trust-anchor-must-be-where-evidence-entered-history-not-any-commit-that-re-introduces-it
artifact: context
---

# Context

Fixes the anchor half of #660, found while adversarially reviewing a proposed slug-only archive
migration — which would have performed the triggering rename across all 161 archives. The defect
is not caused by that migration and outlives it: it is exploitable today by anyone who can land a
commit.

It is also a hard prerequisite. Building the migration first would mean shipping a command whose
entire job is to trigger a known laundering bug.

The issue was filed as "renaming an archive directory launders tampering". That framing was too
narrow — attacking the candidate fixes turned up a third shape with no rename in it at all, and a
fix scoped to the archive path would have left it open. The accurate statement is that any commit
re-introducing a package can become its trust anchor; a rename is merely the cheapest way to
produce one.

The unsigned-approver half of #660 is deliberately **not** here. Adding `actor` to the signed
projection moves `scope_digest`, which is a preimage, and it must not be sold as fixing separation
of duties — it does not.
