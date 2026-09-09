---
change: complete-specsync-6-promotion-and-public-release-contracts
artifact: plan
---

# Plan

1. After scope approval and PR761 merge, update this branch to the merged guidance baseline.
2. Correct the two count guards and obsolete Windows evidence diagnostics; add executable guard regressions and negative controls.
3. Correct CLI help, materialize the approved canonical requirement clarifications, update specs/companions, and preserve runtime behavior.
4. Fix README/CLI examples, SECURITY support wording, and remaining two-platform/publication guidance. Fold6.0 notes without claiming prior publication.
5. Run targeted controls, CLI help/parser checks, site checks, full verify, strict specs/score, pre-push, Trust, and signed provenance verification. Obtain independent and human implementation review, then finalize in the same PR and merge after gates.

After the code change closes: create a fresh annotated RC at the merged tree, qualify Ubuntu/macOS, validate a dry-run dispatch, and obtain an updated independent review. Publish only when release conditions are met; separately verify advertised package channels and hub documentation. Qualification and publication are later release operations, not fabricated pre-merge evidence.
