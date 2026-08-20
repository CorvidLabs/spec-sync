## ADDED

### REQUIREMENT REQ-change-086

A change identity SHALL be minted from its description alone, and identity uniqueness SHALL be enforced directly rather than as a side effect of allocating a number.

Acceptance Criteria
- A newly created change is identified by its description alone, with no allocated number, so two people working from the same base need not coordinate to avoid claiming the same identity.
- A description that would produce an identity already in use is refused by naming the existing change, its location and its state, rather than by exhausting an allocation retry.
- Two workspaces claiming one identity are refused directly, because an allocated number is no longer providing that guarantee as a side effect and an identity that names two packages is ambiguous.
- An identity that carries no number takes part in no number-based accounting, while an identity that carries a malformed number is still refused, because tolerating an absent number must not become tolerating a corrupt one.
- Identities already allocated keep working unchanged, including the historical ones that share a number by prior acknowledgement.
