---
change: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
artifact: design
---

# Design

Persist each spec's validation result alongside its hash, and replay it when the
spec is skipped as unchanged. A skipped spec then contributes exactly what it
contributed when it was last validated: counted in `specs_checked`, warnings
named, verdict unchanged.

The alternative — make the cache skip only re-extraction and always re-validate
— was rejected. It would make every warm run pay full validation cost, which is
the entire point of the cache, and it would not fix the contract: a spec whose
findings are not stored is a spec whose verdict cannot be reproduced by anything
except recomputation.

Worth naming because it recurs: **the snapshot types existed and were unused.**
Someone built the mechanism and never connected it. That is the same shape as
the two dead detectors found in #578 — machinery present, path not taken, and no
test failing because nothing exercised it. Unused infrastructure is not neutral;
it reads as "this is handled" to the next author.
