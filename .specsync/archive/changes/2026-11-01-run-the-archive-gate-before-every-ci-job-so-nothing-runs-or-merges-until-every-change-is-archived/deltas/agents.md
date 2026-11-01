## ADDED

### REQUIREMENT REQ-agents-009

Generated SDD skill and check-command text SHALL describe archive-first CI: a pushed tip with any open change fails the Archive check and runs nothing else, so verification and review happen locally before `change ship`, and CI runs the full lane on the pushed archive tip.

Acceptance Criteria
- Tracked Claude, Codex, Cursor, and Gemini `SKILL.md` files say CI is archive-first and tell the agent to verify locally, record `change review`, run `change ship` without committing in between, then commit and push the archive tip.
- Generated `change check` skill and command text describe `--commit` as local evidence, not product-tip evidence for remote CI.
- No generated text tells the agent to push a product tip and wait for remote CI before review.
- `AGENT_ARTIFACT_TEMPLATE_VERSION` is 6 or higher so `specsync agents install` refreshes copies that teach the product-tip flow.
