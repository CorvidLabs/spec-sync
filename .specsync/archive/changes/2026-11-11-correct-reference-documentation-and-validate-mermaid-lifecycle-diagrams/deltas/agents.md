## MODIFIED

### REQUIREMENT REQ-agents-009

Generated SDD skill and check-command text SHALL describe archive-first CI: a pushed tip with any open change fails the Archive check and runs nothing else, so verification and review happen locally before `change ship`, and CI runs the full lane on the pushed archive tip.

Acceptance Criteria
- Tracked Claude, Codex, Cursor, and Gemini `SKILL.md` files say CI is archive-first and tell the agent to verify locally, record `change review`, run `change ship` or `change finalize` without committing in between, then push the archive commit that the native command creates.
- Generated `change check` skill and command text describe `--commit` as local evidence, not product-tip evidence for remote CI.
- No generated text tells the agent to push a product tip and wait for remote CI before review, or requires an extra manual archive commit after a successful native ship/finalize.
- `AGENT_ARTIFACT_TEMPLATE_VERSION` is 8 or higher so unchanged managed copies refresh to the corrected native-commit guidance; custom artifact protections remain intact.
