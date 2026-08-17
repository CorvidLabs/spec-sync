---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: requirements
---

# Requirements

One requirement is added as a semantic delta. The delta file is the source; `specs/` is
materialized from it rather than hand-edited.

## `deltas/change.md` — REQ-change-069

Declaring an additional affected module SHALL never remove a verification command from what a
change receives.

Stated as a **relation between scopes** rather than as a rule about any particular module, because
the defect was not that one module was mishandled — it was that the selection was evaluated once
per change instead of once per module. A requirement phrased as "an unrouted module must contribute
the project list" would be satisfied by an implementation that still suppressed commands through
some other per-change condition.

## Relationship to the requirement already on the books

`REQ-change-015` states "Reporting mode still executes every configured verification command", and
`REQ-change-058` requires that no lifecycle entry point suppress verification command output. The
code contradicted the first of those — which is precisely the drift this product exists to detect,
present in the product.

REQ-change-069 does not replace REQ-change-015. It adds the property REQ-change-015 assumed and
never stated: that scope declaration cannot be the thing which suppresses a command.

## Explicitly retained behaviour

Two acceptance criteria exist to stop the fix over-correcting:

- A change scoped entirely to routed modules still receives only its component commands. Targeted
  verification is a feature and survives.
- A change declaring no affected module still receives the project-wide list, unchanged.

Without the first, "always append the project list" satisfies the headline requirement and deletes
the feature.
