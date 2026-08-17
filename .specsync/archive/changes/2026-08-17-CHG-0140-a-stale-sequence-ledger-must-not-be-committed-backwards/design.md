---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: design
---

# Design

`floor_sequence_ledger_to_committed(root) -> Result<Option<(u64, u64)>, String>` reads the
ledger committed at HEAD, compares it to the working tree's, and raises the working tree when
it is lower. It returns the pair it changed so the caller can disclose it, and `None` when it
changed nothing.

Called from `git_commit_all`, before `git add -A`, which covers all three lifecycle staging
sites rather than the one named in the report.

## Raise, do not refuse

Refusing would be the more obviously "safe" choice and it is the wrong one. The author did
nothing: their branch sat while `main` moved. A refusal at commit time hands them an error
about a file they never edited, at a step that is not where the problem was introduced.

Raising repairs the state, and returning the old value is what keeps it from being a silent
repair — which is the failure mode this whole release has been closing.

## Only ever upward

`if committed.sequence <= local.sequence { return Ok(None) }` is the control that keeps this
from becoming "always restore the committed ledger". A working tree **ahead** of HEAD is the
ordinary case — it is exactly what `change new` produces — and overwriting it would destroy
the claim the author just made and hand the same ID out twice. Equal marks are not a
divergence and are not reported as one.

## Collisions are merged, not replaced

`acknowledged_collisions` is history, and both sides may hold entries the other does not. The
raise takes the committed list, appends any local entry not already present, and sorts. Taking
either side wholesale would drop acknowledgements.

## Disclosure channel

`eprintln!` rather than the caller's `say`:

- `say` honours `--quiet`. A state correction is not progress chatter and must survive it.
- stdout may be carrying a `--format json` payload. Writing a note there would produce two
  documents on one stream — the exact defect fixed in #443 earlier in this release.

## Failure handling

A committed ledger that cannot be parsed returns `None` rather than an error. It is not
evidence of a higher mark, and the readers that already validate the ledger report it properly
with their own diagnostics. Failing here would replace a good error with a worse one raised
from the wrong place.
