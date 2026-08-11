---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: context
---

# Context

SpecSync 6.0 currently rejects a scope approver who attempts to record the required scoped
review. That is a sound default for teams, but it leaves a solo maintainer unable to complete an
otherwise green lifecycle without inventing a second identity or pausing for an external reviewer.

The maintainer explicitly requested this command shape:

`specsync change review CHG-… --self-review --actor 0xLeif --reason "solo maintainer"`

The exception must be narrow and honest. It replaces only the independent-human identity check
when the self-review actor is the recorded scope approver. It does not bypass definition approval,
targeted verification, commit/digest freshness, ordinary product CI, trust verification, append-only
review history, or same-PR finalization ordering.

Existing v2 independent review evidence must remain readable and valid. New evidence will make its
mode explicit instead of treating a self-review as an independent review or pretending that a
GitHub reviewer check authenticated it. Text and JSON lifecycle status must surface the exception
before an operator decides a change is ready to finalize.

This is the bootstrap change for the feature. The maintainer's authorization is recorded in this
package and the implementation will use the resulting explicit self-review path only after this
change has its normal scope approval, verification, and trust evidence.

Implementation now has an additive review mode under the established v2 record schema: historical
records deserialize as independent, while new records persist either independent GitHub-check
provenance or audited self-review provenance.
Focused CLI, command, and domain regression tests pass before scoped verification.
