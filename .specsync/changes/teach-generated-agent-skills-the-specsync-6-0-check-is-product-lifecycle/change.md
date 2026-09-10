---
id: teach-generated-agent-skills-the-specsync-6-0-check-is-product-lifecycle
state: implementing
type: documentation
base_commit: 6f11e34ef65710dd8839bfaaa6e6bc9004d6a530
---

# Teach generated agent skills the SpecSync 6.0 check-is-product lifecycle

## Intent

Teach generated agent skills the SpecSync 6.0 check-is-product lifecycle

## Affected Canonical Specs

- `agents`

## Acceptance Criteria

- Generated Claude/Cursor/Codex/Gemini SKILL.md files lead with specsync check as the product and SDD opt-in via change adopt.
- Generated create-change/check/audit commands use 6.0 happy-path verbs: approve --actor, check --commit, review --reviewer, ship/finalize; v1 start/verify/accept/archive are labeled recovery only.
- AGENT_ARTIFACT_TEMPLATE_VERSION is 5 so specsync agents install refreshes stale host skills.
- cargo test agents:: passes.
- specsync check --spec agents passes.

## No-spec Rationale

Not applicable
