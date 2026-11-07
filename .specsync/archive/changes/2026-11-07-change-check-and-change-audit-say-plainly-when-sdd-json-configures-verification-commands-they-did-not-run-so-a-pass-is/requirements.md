---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: requirements
---

# Requirements

- Text `change check` (plain and `--commit`) and `change audit` print one stderr note naming the number of configured commands SpecSync did not run, with correct singular/plural wording and how to stop the note.
- JSON check and audit output carries `configured_commands_not_run` beside `verification` or `report`: top-level commands, component commands in module order, then strict commands, each once; the key is present as `[]` when nothing is configured.
- The disclosure is additive only: it never changes success or failure and never starts a configured command.
- `change check --commit --json` keeps its existing empty success stream (#839).
- This repository empties only `verification_commands`; its 13 component modules and 3 strict commands remain configured and disclosed.
- Existing integration fixtures that assert empty stderr are updated for the additive note, while unrelated behavior remains unchanged.
