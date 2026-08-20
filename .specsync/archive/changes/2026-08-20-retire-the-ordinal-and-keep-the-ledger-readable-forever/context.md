---
change: retire-the-ordinal-and-keep-the-ledger-readable-forever
artifact: context
---

# Context

The last step of the change-identity work, and the first change this repository has created
with a slug-only identity — this package is `.specsync/changes/retire-the-ordinal-and-keep-the-ledger-readable-forever/`,
with no `CHG-NNNN` anywhere in it.

Steps 1-4 turned out to have done most of the work. On the previous binary a hand-converted
slug-only workspace already passed `list`, `status`, `show`, `answer`, `approve`, `check`,
`review` and `finalize`, archiving with a clean acceptance manifest. Identity from `state.json`
(CHG-0159), a capped and guarded slug (CHG-0161), a prefix-free `validate_change_id` (CHG-0162)
and succession on `created_at` (CHG-0160) between them carried it end to end.

What remained was smaller than expected in the lifecycle and larger than expected in the
guarantees. The ordinal was not only a name prefix: it was the repository's only assurance that
two changes could not share an identity, and the numeric collision gate was enforcing that as a
side effect. Removing it without a replacement makes two same-named packages report
`✓ audit passed`.

The owner decided that historical archives keep their `CHG-NNNN-slug` identities permanently.
There is no migration and no rewriting of old packages — they are inert history. That decision
removed the riskiest work in the plan, and it is what makes this change a retirement rather than
a migration.
