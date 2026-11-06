---
change: describe-the-cli-recursion-guard-as-refusing-an-inherited-verification-context-not-a-specsync-made-child
artifact: design
---

# Design

| Clause | Was | Becomes |
|---|---|---|
| REQ-cli-003 body | a configured verification child re-enters the lifecycle | a check/change/lifecycle command that runs under an inherited `SPECSYNC_VERIFICATION_CONTEXT` |
| REQ-cli-003, new criterion | (none) | SpecSync sets the marker on no child of its own (REQ-change-023) |
| cli spec Purpose, `cli.3md` context | blocks configured verification children | refuses commands under an inherited marker; SpecSync starts no verification child |
| docs/HLD.md failure-mode row | a verification child calling check or change | a check or change that inherits the marker from a caller |

The requirement and Purpose change through the semantic delta. The 3md context plane, the HLD, the lesson
plane and the CHANGELOG are direct edits inside the declared paths.
