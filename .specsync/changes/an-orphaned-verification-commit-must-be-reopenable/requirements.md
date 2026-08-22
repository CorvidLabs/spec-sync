---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: requirements
---

# Requirements

No new requirement ID. This change amends four existing ones, because the behaviour it enables was
explicitly forbidden by the requirements as written — a living requirement describing behaviour the
code no longer has is exactly the drift this tool exists to catch.

## Amended

**REQ-change-017** — the refusal criterion now qualifies "exact or successor-covered" with "AND
still anchored in current history". Previously it forbade the admitted case outright.

**REQ-change-018** — the criterion "An unreachable verification commit is allowed only when the
exact accepted-transition anchor or explicit predecessor/path/module/digest successor evidence is
provable from trusted history" directly contradicted this change. It now names unreachability as an
admissible staleness axis in its own right, while stating that it never substitutes for the
succession evidence REACCEPTANCE requires and that every other authentication check stays fatal.

**REQ-change-034** — gains the governing principle: reopen admits exactly the axes for which no
restore exists. Content that drifted can be put back; an orphaned commit can only be superseded.

**REQ-change-035** — gains the criterion that an anchor-caused reopen records identical stale and
current digests and still preserves immutable sequence history, and that the recorded cause — not
digest inequality — is what proves staleness.

## Invariants amended

**15** — reopening is rejected only when inputs are current AND the commit is still anchored.

**18** — rewritten from "accepts unreachable verification commits only when canonical acceptance is
recorded..." to name unreachability as its own axis with a recorded cause.

This amendment was approved as a deliberate policy decision, not inferred. The security argument is
in `context.md`: reopen never gated a capability an attacker lacks, and the authorization that
matters is downstream and untouched.
