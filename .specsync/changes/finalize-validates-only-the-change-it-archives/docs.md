---
change: finalize-validates-only-the-change-it-archives
artifact: docs
---

# Docs

`specs/change/change.spec.md` documents `audit_project_with_strict` in its Public API table. `specs/cmd_change/cmd_change.spec.md` invariant 4 says that `change audit --strict` also re-walks every archived change's own workflow-version history, which `change finalize` no longer repeats. REQ-change-124 in `specs/change/requirements.md` states the contract. Neither the site docs nor the CLI help change: `--strict` is already a global flag that `change audit` accepts.
