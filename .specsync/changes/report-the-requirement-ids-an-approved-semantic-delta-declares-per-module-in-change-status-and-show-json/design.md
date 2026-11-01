---
change: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json
artifact: design
---

# Design

## Shape

A new top-level key in single-record change JSON (`print_record`, `print_mutation_record`):

```json
"approved_deltas": [
  {
    "module": "shipping",
    "delta_intact": true,
    "requirements": { "added": ["REQ-shipping-002"], "modified": [], "removed": [] },
    "sections": { "added": [], "modified": ["Public API"], "removed": [] },
    "materialized": false
  }
]
```

- The array is always present. It is `[]` whenever the domain reports nothing, so a consumer can
  tell "this SpecSync predates the field" (absent) from "nothing approved to report" (`[]`).
- Entries are sorted by module, and every key list is sorted and deduplicated.
- `delta_intact` is `true`, `false` or `null`. `requirements`, `sections` and `materialized` are
  serialized only when `delta_intact` is `true` (`skip_serializing_if` on an `Option`). A body
  that does not match its approval, or whose approval is unknown, has no "approved" IDs to report.
  Leaving the lists out, rather than sending empty ones, means `requirements.added == []` can never
  be read as "approved, adds nothing".

## Domain types (`src/change/sdd_version.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaDeclarations { pub added: Vec<String>, pub modified: Vec<String>, pub removed: Vec<String> }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedDeltaSummary {
    pub module: String,
    pub delta_intact: Option<bool>,          // serialized as null when None
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<DeltaDeclarations>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<DeltaDeclarations>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized: Option<bool>,
}
```

## Domain function (`src/change/collect_requirement_ids.rs`)

`pub fn approved_delta_summaries(root: &Path, record: &ChangeRecord) -> Vec<ApprovedDeltaSummary>`

1. Return `[]` unless `record.state` is Approved, Implementing or Verifying, the record is not
   `no_spec_change`, and `ensure_definition_approval_valid(root, record)` is `Ok`. This is the same
   gate `approval_currency` uses. A stale scope means no current approval covers any delta, and
   `summary.approval_valid` and `scope_expansion` already report it.
2. Refactor the body of `changed_approved_delta_modules` into a private
   `approved_delta_binding(root, record) -> Result<DeltaBinding, String>`, where `DeltaBinding` is
   `Unrecorded`, or `Recorded { per_module: BTreeMap<module, (intact: bool, body: String)> }`.
   - It reads each delta once with `read_bounded_change_text` and digests the CRLF-folded body
     through the existing `canonical_delta_body` framing. A module the approval never recorded
     counts as not intact, as it does today.
   - `changed_approved_delta_modules` becomes a thin projection of it: modules with
     `intact == false`, or the same `Err`. Its outputs and the refusal text of
     `ensure_approved_delta_bodies_unchanged` stay byte-identical. Existing #704, #719 and #730
     tests pin them.
   - `delta_body_digests` keeps its signature for approval-time recording. The new helper shares
     its per-module framing through a small `delta_body_digest(module, body)` so the two cannot
     drift.
3. Map to entries:
   - `Err(_)`, meaning a withdrawn binding or an unreadable or unsafe delta: every
     `record.affected_specs` module gets `delta_intact: Some(false)` and nothing else. This matches
     `approval_currency`, which reads the same `Err` as `approval_valid: false`.
   - `Unrecorded`: every module gets `delta_intact: None` and nothing else.
   - `Recorded` with `intact == false`: `Some(false)` and nothing else.
   - `Recorded` with `intact == true`: run `parse_delta(&body)` on the same string and split the
     items by target and operation into `requirements` and `sections`. A parse failure maps to
     `Some(false)` with no lists; approval refused such a body, so it cannot be the approved one.
     `materialized` is set as follows.
4. `materialized` for intact modules:
   - `false` when `!record.canonical_applied`. There has been no materialization, and on a first
     run `prepare_pending_delta_application` may raise refusals that only a first application
     should raise.
   - Otherwise one in-memory `prepare_pending_delta_application(root, record)` call (no writes):
     `true` exactly when the module is not in `pending`. `Err` maps every intact module to
     `false`. Status cannot confirm the tree, and `false` is the conservative answer.

## Consistency invariants

- For every module, `delta_intact == Some(false)` if and only if the module is in
  `summary.changed_approved_deltas`, or the binding was refused as a whole (when
  `approval_valid` is false and `changed_approved_deltas` is empty).
- `summary.approval_valid == true` in an active state implies that no entry has
  `delta_intact == Some(false)`.

Unit tests assert both invariants on every fixture, so the two projections cannot drift.

## Command layer (`src/commands/change.rs`)

`print_record` and `print_mutation_record` add `"approved_deltas": change::approved_delta_summaries(root, record)`
to their `json!` objects. That is the only edit. Text paths, `print_roster`, `print_transition*`,
`ChangeSummary` and the MCP tools are untouched. The values are module names, requirement IDs,
section names and booleans. No digest reaches any sink.

## Rejected alternatives

- Placing it in `ChangeSummary`: rejected in `research.md`. It would also reach the roster and
  audit and read canonical specs once per listed change.
- Reporting IDs whenever the body parses, with `delta_intact` beside them: this breaks the
  issue's own rule ("never report a requirement as approved when the delta body changed after
  approval") for any consumer that reads the list and skips the flag.
- Publishing the digest construction so consumers can compare it themselves: this freezes an
  internal that changed in #730. The issue asks for the opposite.
- Recording a `materialized_delta_digests` field: rejected for #741 already. It is a live
  `ChangeRecord` field that moves four definition-digest paths.

## Observable changes and compatibility

| Surface | Before | After |
|---------|--------|-------|
| `change status <id> --json`, `change show <id> --json`, `change new/answer/depend/supersede --json` | no `approved_deltas` key | `approved_deltas` array, always present |
| `summary.*`, `change list --json`, `change status --json` without an ID, `change approve --json` | unchanged | unchanged |
| Text output | unchanged | unchanged |
| Persisted files, digests, ledgers, materialization, gates | unchanged | unchanged |

The change is additive JSON. A strict-schema consumer that rejects unknown top-level keys would
see a new key. This JSON has no published schema. `acceptance_entries` (#503) and `briefing`
(#804) were added to the same object in the same way. It is still classified as a public-contract
change, because tools such as Chough will depend on the field's meaning.
