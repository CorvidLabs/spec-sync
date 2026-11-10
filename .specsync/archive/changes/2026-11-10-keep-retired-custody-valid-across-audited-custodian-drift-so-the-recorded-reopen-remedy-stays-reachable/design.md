---
change: keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable
artifact: design
---

# Design

`validate_retirement_with_records` replaces its `retirement_custody` recompute and the
closing/manifest digest equality checks with `retirement_custody_holds`:

1. The replacement must exist, differ from the retired change, be Accepted or Archived, and
   not depend on the retired change (unchanged rules).
2. The replacement's recorded accepted evidence is authenticated
   (`AcceptedInputValidationMemo::authenticate`): genuine, anchored, internally bound. No
   live-tree freshness recursion.
3. For every receipt custody entry: the frozen prior manifest (itself pinned by
   `prior_manifest_digest`) carries the path at the recorded predecessor digest and owners;
   the recorded classification matches the receipt's own fields (`same-signed-owners` means
   equal owner vectors; `unchanged-exact-file-current-owner` means the recorded predecessor
   owners are exactly `@exact:delivery`, the recorded digests are equal, and the recorded
   replacement owners are non-empty named modules); the custodian's current authenticated
   manifest carries the path under exactly the recorded replacement owners.

`retire_change_with_source` keeps calling `retirement_custody` unchanged: a fresh custodian,
custody recomputed from current evidence, classifications derived under current rules.
