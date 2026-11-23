---
change: in-a-shallow-clone-name-git-fetch-unshallow-origin-in-every-lifecycle-history-refusal-and-refuse-change-new-before
artifact: design
---

# Design

- **One shallow test**, `git rev-parse --is-shallow-repository`, asked only when a lifecycle-history check has already refused, so a full clone spawns nothing new.
- **The baseline history check** (`validate_workflow_v2_baseline_history`, through `validate_workflow_v2_baseline`): in a shallow clone, any refusal becomes the shallow message, naming the cutoff and `git fetch --unshallow origin`.
- **The correction-ledger text** (`load_change_for_definition_mutation`, `ensure_text_correction_ledger_valid`): in a shallow clone, the same shallow message.
- **`change new`** asks the same history check before it allocates a workspace, only in a shallow clone of an adopted project, and refuses with the `change new` form of the message, writing nothing (REQ-change-108's order).
- **A full clone** keeps every message as it is.
