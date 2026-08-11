---
change: CHG-0101-add-audited-solo-maintainer-self-review-override
artifact: research
---

# Research

The current domain has a single `ScopedReviewRecord` schema version 2, fixed
`github_actions_check` provenance, and an unconditional rejection when reviewer equals the
definition approver. Validation occurs at recording time, when loading append-only history, when
determining review freshness, and before finalization. The implementation therefore needs one
central mode-aware validator reused by each of those paths; changing only the CLI rejection would
leave finalization and historical validation inconsistent.

The command parser currently requires `--reviewer`, so self-review needs an explicit alternate
input branch rather than interpreting a reviewer value equal to the approver as consent. Existing
records use `#[serde(deny_unknown_fields)]`; additive fields must have defaults so v2 committed
evidence remains compatible.

The trust gate remains independent of review provenance. `fledge trust verify` and the configured
verification commands will stay required evidence for this change and for any later self-reviewed
change.
