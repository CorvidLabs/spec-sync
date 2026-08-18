## ADDED

### REQUIREMENT REQ-cmd-stale-004

`stale` SHALL NOT report a spec as current when a file it cites no longer exists, and SHALL NOT claim an all-clear over any spec it could not measure.

Acceptance Criteria
- A cited file that git knew at the spec's baseline and that is now gone makes the spec stale regardless of the drift threshold, because a deletion measures a single commit and any threshold above one would otherwise bury it.
- A cited path git never knew is reported as unmeasurable rather than as evidence of freshness, and the command exits non-zero unless the project's enforcement is warn.
- A spec that measured some of its files and not others discloses the ones it could not, so the per-file breakdown is not read as exhaustive.
- The all-clear line is withheld whenever anything went unmeasured, in every output format rather than only the human one.
- The machine-readable form carries the same distinctions as the human one, so a consumer reconciling totals does not absorb unmeasurable specs into either the stale or the fresh count.
