---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: context
---

# Context

Four accepted changes (the CHG-0048 musl build, CHG-0051, CHG-0052, and CHG-0053) were verified
and reaccepted on side branches that were then squash-merged into `main`. Squash merge preserved
their final evidence bytes in `main` commits but discarded the original acceptance-transition
commits, and the verification commits named in `verification.json` are not ancestors of `HEAD`.

`authenticated_accepted_transition` only trusts first-acceptance transition anchors, and the
staged-snapshot path additionally requires the verification commit to be in current history. As a
result `specsync change archive` fails closed with `requires exactly one trusted transition
matching its state, verification, and closing evidence; found 0` for all four changes, even though
a commit on `main` records each change as accepted with byte-identical `state.json`,
`verification.json`, and `approvals.json`, and `specsync change check` validates all four as
`exact`.

Projects that squash-merge pull requests (the default for this repository) will hit this whenever
accepted evidence is refreshed while a change is already accepted and then squash-merged, so the
trust model must recognize an in-history accepted record as authenticating evidence regardless of
whether that record is the first acceptance transition.
