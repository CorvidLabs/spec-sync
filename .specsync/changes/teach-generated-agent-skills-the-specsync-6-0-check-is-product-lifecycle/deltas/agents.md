## ADDED

### REQUIREMENT REQ-agents-007

Generated SDD skill and command text SHALL lead with `specsync check` as the product and SHALL
describe the verified change workflow as opt-in via `specsync change adopt`. When that workflow
is adopted, generated prose SHALL teach the 6.0 happy path and SHALL label v1 lifecycle verbs as
recovery only.

Acceptance Criteria

- Tracked Claude, Codex, Cursor, and Gemini `SKILL.md` files contain `` `specsync check` is the product ``
  and `specsync change adopt`.
- Those files teach `approve --actor`, `check --commit`, `review --reviewer`, and `ship`/`finalize`.
- Those files label `start` / `verify` / `accept` / `archive` as v1 recovery, not the happy path.
- Generated create-change, check, and audit commands use the same happy-path verbs.
- `AGENT_ARTIFACT_TEMPLATE_VERSION` is 5 or higher so `specsync agents install` refreshes stale copies.
- A unit test fails if tracked `SKILL.md` files drop the check-is-product sentences.
