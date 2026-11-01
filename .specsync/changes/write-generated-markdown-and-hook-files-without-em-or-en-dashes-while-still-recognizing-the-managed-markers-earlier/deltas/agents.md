## ADDED

### REQUIREMENT REQ-agents-009

Generated agent skill and command files SHALL contain no U+2013 (en dash) or U+2014 (em dash).

Acceptance Criteria
- `agents install` for every tool writes no U+2013 or U+2014.
- `AGENT_ARTIFACT_TEMPLATE_VERSION` is 6 or higher, so an installed copy whose bytes still match its recorded digest is refreshed and a customized copy is reported as a conflict, as before.
- The tracked Claude, Cursor and Gemini command assets and `.specsync/agent-artifacts.json` in this repository match the generated output.
