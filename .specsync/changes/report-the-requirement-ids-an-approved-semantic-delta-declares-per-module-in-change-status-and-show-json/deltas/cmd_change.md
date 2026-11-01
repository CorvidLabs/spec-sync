## ADDED

### REQUIREMENT REQ-cmd-change-023

Every change JSON output that renders a single change record SHALL carry the domain's per-module approved delta declarations under a top-level `approved_deltas` array, rendered verbatim from `change::approved_delta_summaries`, and text output SHALL NOT change.

Acceptance Criteria
- `specsync change status <id> --json` and `specsync change show <id> --json` include a top-level `approved_deltas` key in every state; it is `[]` when the domain reports nothing, so a reader can tell a release without the field from a change without approved deltas.
- The same key appears in every other JSON output that renders the single-record shape: `change new`, `change answer`, `change depend` and `change supersede`. Those outputs render `DefinitionMutationResult.approved_deltas`, built under the project lock together with `summary`, and compute nothing after persistence; `change depend` on an approved change carries the same entries `change status` reports. Transition outputs such as `change approve --json`, and the roster printed by `change status` without an ID, do not change.
- Each entry carries `module` and `delta_intact`; `requirements`, `sections` and `materialized` appear only when `delta_intact` is true.
- No existing key changes name, type or value; `summary` and `change list --json` gain nothing.
- Text `status` and `show` output is unchanged and prints no requirement list.
