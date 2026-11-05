---
change: prepare-specsync-6-2-0-package-metadata-and-release-handoff
artifact: context
---

# Context

This is user-authorized release preparation only for spec-sync, delegated by the owner's session. 6.2.0 is a minor release because #846 adds the `change send-back` command. Existing source branches and historical approval records are preserved. The release handoff names exact source and independent publication gates.

The branch was cut from main 61e8ca6a, which already carries #836, #837, #838, #840, #841 and #853. #842, #845 and #846 were open when this was written; their CHANGELOG and handoff entries are included and marked pending merge. They must be on main and merged here (no rebase), or their entries removed, before the RC tag.
