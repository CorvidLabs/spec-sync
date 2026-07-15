---
change: CHG-0041-synchronize-generated-create-spec-agent-commands-with-the-corrected-free-text-pa
artifact: requirements
---

# Requirements

### REQ-CHG-0041-001

Checked-in create-spec commands SHALL classify the complete non-flag input before deriving a module
name.

Acceptance Criteria

- Claude, Cursor, and Gemini commands remove each standalone `--minimal` flag before classification.
- A single valid identifier remains unchanged.
- Quoted or unquoted free text derives and confirms a short deterministic kebab-case slug and never
  uses only the first word.
- No checked-in command contains the retired first-whitespace-token instruction.

### REQ-CHG-0041-002

Generated guidance SHALL make flag position behavior explicit for both supported input classes.

Acceptance Criteria

- Examples cover `--minimal billing`, `billing --minimal`, `--minimal I need CSV export`, and
  `I need CSV export --minimal`.
- Both bare-module examples preserve `billing`.
- Both free-text examples preserve the complete description and derive `csv-export`, not `I`.

### REQ-CHG-0041-003

The repository SHALL detect drift between checked-in native agent commands and current installer
templates.

Acceptance Criteria

- A deterministic test installs Claude, Cursor, and Gemini assets into a clean project.
- The rendered create-spec commands match the repository's corresponding checked-in assets exactly.
- Reinstallation remains idempotent after the comparison.
