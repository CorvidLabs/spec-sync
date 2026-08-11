---
id: CHG-0101-add-audited-solo-maintainer-self-review-override
state: implementing
type: feature
base_commit: f1e7d3abea2fed4f12da77a0c700e55d90bd2ad7
---

# Add audited solo-maintainer self-review override

## Intent

Add audited solo-maintainer self-review override

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- The review command SHALL accept a solo-maintainer exception only when --self-review, --actor, and a non-empty --reason are supplied and the actor matches the approved scope approver; it SHALL persist the exception, actor, and reason in durable review evidence, visibly mark the status as self-reviewed, preserve ordinary independent-review enforcement by default, and reject malformed or mismatched identities.

## No-spec Rationale

Not applicable
