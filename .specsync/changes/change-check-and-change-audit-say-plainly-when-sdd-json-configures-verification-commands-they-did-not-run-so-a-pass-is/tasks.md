---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: tasks
---

# Tasks

- [x] Capture SHIP-1.c.3 and write failing text, JSON, empty-policy, no-spawn, and repository-policy probes on b3ca5f97.
- [x] Narrow the repository pin to Leif's 3b decision: empty only `verification_commands`, retaining 13 component modules and 3 strict commands.
- [x] Implement the shared configured-command disclosure on plain/committed check and audit, with ordered deduplicated JSON lists.
- [x] Empty only the top-level policy list and update cmd_change contract/companions, site configuration, ADOPTING, HI provenance, INTENT, and CHANGELOG.
- [x] Run the focused probes and full integration suite, strict spec checks, `fledge lanes run verify`, and `fledge trust verify`.

Human gates, not tasks: Leif's scope approval and later implementation review, both relayed by orc t18. Chough case runs and evidence recording are verification steps, not checkboxes.
