---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: requirements
---

# Requirements

Semantic deltas carry the SHALL statements:

- `REQ-cmd-check-016` fence-aware `--fix` writes the original section, not the blanked copy
- `REQ-git-utils-005` sanitized git children without a ceiling that blocks parent-repo discovery
- `REQ-cmd-new-001` canonical spec body matches `generate_spec` / scaffold name rules
