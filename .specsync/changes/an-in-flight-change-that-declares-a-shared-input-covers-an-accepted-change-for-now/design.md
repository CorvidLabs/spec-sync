---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now
artifact: design
---

# Design

- **Covered for now.** In `validate_accepted_inputs_recursive`, a changed input that no accepted or
  archived successor covers is covered for now by the first approved, implementing or verifying
  change created after the accepted one that declares the input's owner module and a scope
  covering its path (`in_flight_coverer`). The walk returns `ProvisionallyCovered` and keeps each
  `(path, change)` in its memo.
- **Reported.** Terminal evidence reads `provisionally-covered` with a reason naming each path and
  its covering change. `change audit` adds a warning, not an error. Doctor treats it as covered,
  matching the archive preflight, which passes it.
- **May archive.** `change archive` takes the archived change's own coverage from the closing
  check it already runs (`ensure_archiving_closing_approval_valid`, against the structural roster)
  and writes `coverage.json` (`covered_by: [{path, change}]`) into the package with its
  `state.json` and `change.md`. Archived integrity does not read it. A first version walked the
  change again over the checked roster, which re-walked every archive's history: finalize grew by
  44 Git processes per archive and the own-bytes test failed. Reusing the closing check keeps
  finalize at 290 spawns, flat across archives.
- **Not settled.** A provisionally covered change is not offered as another change's covering
  successor, nor accepted as a retirement replacement, because its own currency is only deferred.
  Reopen stays allowed for it.
- **Afterwards.** When the covering change is accepted, ordinary successor coverage takes over; if
  it is closed, nothing covers the input and it is stale again.

Rejected: downgrading every stale accepted input to a warning (the issue's broader option), which
would hide inputs nobody is working on.
