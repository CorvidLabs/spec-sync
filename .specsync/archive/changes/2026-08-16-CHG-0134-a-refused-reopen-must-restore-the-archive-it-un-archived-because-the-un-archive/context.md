---
change: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
artifact: context
---

# Context

This was the only confirmed unrecoverable bug in 6.0.

`reopen_change` moved the dated archive package into `.specsync/changes/` and
THEN ran roughly ten preconditions — approval validity, verification evidence,
contract digest, execution digest, stale-versus-current delivery inputs,
successor coverage. On a healthy tree with no drift, every one of those returns
`Err`. So a CORRECTLY REFUSED reopen consumed the archive:

    finalize, then reopen with NO drift
    -> error: accepted change delivery inputs are current …
    -> rc=1
    -> archive 1 -> 0, orphan at .specsync/changes/<id>, state.json still "archived"

Retry then failed differently — `cannot un-archive …: an active change directory
already exists` — which is the signature of the first attempt having eaten the
package. `list`/`status` still said archived; `check` and `finalize` refused
because archived; `ship` said already archived. Every recovery verb refused.

If the archive tip had been committed, `git restore` plus deleting the orphan
reconstructed it — and then reopen failed with `ambiguous active/archive
workspace locations`. **If the archive was never committed, the package was
gone.**

The refusal itself was right. The destruction that preceded it was the bug.
