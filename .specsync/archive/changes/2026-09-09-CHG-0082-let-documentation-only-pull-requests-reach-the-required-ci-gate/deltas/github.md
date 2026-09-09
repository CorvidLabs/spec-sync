## MODIFIED

### SPEC SECTION Invariants

Every path that can be merged can reach the required CI gate; a path the CI
workflow cannot trigger can never report the gate and blocks its pull request.

Release qualification verifies exactly the tag protections this repository actually has, and names
every protection it does not verify on every run, green runs included. A gate that demands an
unprovisioned policy fails on every candidate and therefore verifies nothing — it is not a safe
default, because the protections that DO exist are never reached. Dropping a check from the gate is
permitted; dropping it silently is not. The tag protections that remain admit no bypass actor and
no broadening — where that can be observed. GitHub returns `bypass_actors` only to a caller with
admin access to repository settings, and the workflow token is not one, so the field is ABSENT
from every payload CI fetches. Absence means UNOBSERVED, never "no bypass actors": it is checked
when visible, refused when it grants anyone, and named in the unenforced disclosure when it cannot
be read. Requiring it made the gate impossible to satisfy from CI, which is how a lane stayed red
on every candidate while appearing to enforce something.

Release authority is stated wherever it is exercised. The final tag is created by the release
workflow's own token under a permission scoped to the single job that writes it, so the authority
to run the release lane is the authority to create a release tag; that equivalence is announced by
every run and recorded at the job itself, never left to be inferred from a green result. A named
deployment environment that does not exist is not a gate — GitHub materializes it unprotected on
first use — so the workflow names no environment rather than publish a gate that gates nothing.
