---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: docs
---

# Docs

`CHANGELOG.md` records the user-visible behaviour change: `specsync check` no
longer exits non-zero because of SDD lifecycle state. Repositories that were red
for trust-layer reasons alone now exit 0 and report the active-change count as
information.

No README or site changes: the documented purpose of `check` (validate specs
against source) is unchanged; only the extra gate is removed.

Internal only: removing `check_project_quiet` and `ConfiguredCommandOutput` changes no
public API and needs no user-facing documentation beyond the CHANGELOG entry above.
