---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: docs
---

# Docs

`site/src/content/docs/cli.md` gets one paragraph after the existing `summary.approval_valid`
paragraph. It says:

- `change status <id> --json` and `change show <id> --json` carry a top-level `approved_deltas`
  array. Each entry has `module`, `delta_intact`, and, only when `delta_intact` is `true`,
  `requirements` and `sections` (`added`, `modified` and `removed` keys taken from
  `### REQUIREMENT` and `### SPEC SECTION` headings) and `materialized`. When `delta_intact` is
  `true` but the current delta grammar refuses the body (an approval recorded under an older
  grammar), the entry carries `declarations_error` instead of those keys.
- `change answer`, `depend` and `supersede --json` carry the same key, built in the same
  transaction as their `summary`. `change new --json` carries it too, always `[]`, because a new
  change is a draft.
- `delta_intact` is `false` when the delta body changed after approval, for the same reason
  `change check` refuses it. It is `null` when the approval predates the delta binding, which
  means unknown, not tampered.
- `materialized` is `true` once the canonical spec carries the approved body. It is `false` again
  after a re-approved correction until the next `change check`.
- The array is `[]` outside approved, implementing and verifying states, and when the approved
  scope is stale.
- A tool that links to a requirement not yet in `requirements.md` should accept it only while
  `summary.approval_valid` is true and the module's `delta_intact` is `true`, and should label it
  pending until `materialized` is `true`.

The paragraph includes a short example JSON block (the `shipping` entry from `design.md`).

Other docs:

- After verification, add a lesson on a later plane of `specs/change/change.3md` and of
  `specs/cmd_change/cmd_change.3md`. One point for each:
  - The IDs and the integrity verdict must come from the same bytes.
  - A list is withheld, not emptied, when its provenance is not proven.
  - Two projections of one input in one JSON object must share one read, or they can disagree.
- Canonical requirement and Public API texts arrive through the approved deltas.
- `CHANGELOG.md` is written by the release change. The PR body states the new JSON key so the
  release notes can carry it.
