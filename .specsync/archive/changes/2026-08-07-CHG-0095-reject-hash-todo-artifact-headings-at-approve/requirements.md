---
change: CHG-0095-reject-hash-todo-artifact-headings-at-approve
artifact: requirements
---

# Requirements

Approve and artifact completeness must reject placeholder bodies that are only
markdown TODO headings (for example a heading titled TODO) or bare TODO lines,
matching the existing HTML TODO-comment rejection.

Real prose or completed checklists remain acceptable.
