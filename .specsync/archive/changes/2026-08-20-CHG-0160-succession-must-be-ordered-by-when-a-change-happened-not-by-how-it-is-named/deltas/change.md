## ADDED

### REQUIREMENT REQ-change-082

Succession SHALL be ordered by when a change was created rather than by how it is named, and every ordering applied to a change's succession edges SHALL agree with the ordering that is signed.

Acceptance Criteria
- A superseded change that was created after its successor is refused whatever the two are called, because succession is a claim about what happened first and a name is not evidence of that.
- Succession ordering does not read a number out of an identifier, so an identifier that carries no number cannot silently reduce the relation to alphabetical order.
- Every sort applied to a change's succession edges produces the same order as the sort whose result is signed, so a canonical form cannot be rejected by the gate that validates it.
- Changes created in the same second remain strictly ordered, because the surrounding gates enforce strict sorts and a tie would make a valid record unrepresentable.
