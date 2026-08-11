---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: requirements
---

# Requirements

The solo-maintainer exception is a narrowly-scoped replacement for an unavailable independent
reviewer, not a second approval mechanism and not an authentication claim.

The domain SHALL require an explicit self-review mode, a valid stable actor equal to the approved
scope approver, and a non-empty reason. It SHALL persist the mode, actor, reason, verdict, and
current commit/digest bindings in append-only review evidence. Ordinary reviews remain independent
by default and retain their required hosted-check provenance. A self-review SHALL not report that
it received independent or hosted-review authentication.

Only a current passing self-review may satisfy the scoped-review gate. Definition approval,
targeted verification, freshness validation, product CI/trust gates, and same-PR finalization remain
mandatory. Legacy independent review evidence remains valid and readable.

The command adapter and status surfaces SHALL clearly distinguish an independent review from an
audited self-review, including the self-review actor and recorded reason in machine-readable output.
