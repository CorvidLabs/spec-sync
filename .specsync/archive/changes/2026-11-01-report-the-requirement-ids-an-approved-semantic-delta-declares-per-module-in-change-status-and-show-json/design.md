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
- `delta_intact` answers one question only: does the body on disk match the digest the approval
  recorded (CRLF folded)? It is `true`, `false` or `null` (no definition approval in the ledger
  recorded a digest).
- `requirements`, `sections` and `materialized` are serialized only when `delta_intact` is `true`
  AND the current delta grammar parses that body. A body that does not match its approval, or
  whose approval is unknown, has no "approved" IDs to report. Leaving the lists out, rather than
  sending empty ones, means `requirements.added == []` can never be read as "approved, adds
  nothing".
- `declarations_error` (string) is serialized only when `delta_intact` is `true` and `parse_delta`
  refuses the body. Approval runs the same parser, so on a ledger written by this binary this
  needs an approval recorded under an older, more permissive grammar, or a later stricter one.
  The body is still the approved body, so `delta_intact` stays `true` and the binding invariants
  below still hold. Only the declarations are unavailable, and the field says why. The text is the
  `parse_delta` diagnostic (a heading and a fixed sentence). It never carries a digest.

So each entry is exactly one of four shapes:

| Shape | Keys present |
|-------|--------------|
| intact and parsed | `module`, `delta_intact: true`, `requirements`, `sections`, `materialized` |
| intact, grammar refuses the body | `module`, `delta_intact: true`, `declarations_error` |
| not intact | `module`, `delta_intact: false` |
| unknown | `module`, `delta_intact: null` |

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declarations_error: Option<String>,
}
```

`DefinitionMutationResult` (crate-private) gains `approved_deltas: Vec<ApprovedDeltaSummary>`.

## One binding pass per invocation

Today the delta files are read by three separate paths that a status call can reach:
`approval_currency -> changed_approved_delta_modules -> delta_body_digests` (for
`summary.changed_approved_deltas` and `approval_valid`), and `prepare_pending_delta_application`
(materialization). `summarize_change_with_strict` may also run more than once per command.
Adding a fourth independent read would let `summary.changed_approved_deltas` and
`approved_deltas[].delta_intact` disagree in one JSON object if a delta changes between reads.
The design removes that by construction rather than asserting it with static fixtures:

1. `approved_delta_binding(root, record) -> Result<DeltaBinding, String>` (private, in
   `src/change/collect_requirement_ids.rs`) is the only reader of delta bodies for the binding.
   `DeltaBinding` is `Unrecorded`, or
   `Recorded { per_module: BTreeMap<String, BoundDelta { intact: bool, body: String }> }`.
   It loads the ledger and effective approval exactly as `changed_approved_delta_modules` does
   today, reads each delta once with `read_bounded_change_text`, and digests the CRLF-folded body
   through a new `delta_body_digest(module, body)`, which `delta_body_digests` also uses, so the
   approval-time and status-time framing cannot drift. A module the approval never recorded
   counts as not intact, as today. `Err` texts are the existing ones.
2. The result is memoized per change ID in a delta-binding memo, a thread-local stack with a
   drop guard (`DeltaBindingMemo`). Every `ChangeReadScope` holds one, so `change status`,
   `change show`, `change list` and `audit_project` (all of which already install a read scope)
   read each delta once per invocation. `definition_mutation_result` installs its own guard while
   it builds `summary`, `strict_summary` and `approved_deltas` under the project lock. The
   refusing gates install none: the `ensure_approved_delta_bodies_unchanged` calls in
   materialization (`src/change/sdd_version.rs:2926`) and acceptance
   (`src/change/historical_definition_digest_matches.rs:2222`) run outside any memo, so they stay
   live, as `ChangeReadScope`'s contract requires for mutating APIs. The audit report's call
   (`historical_definition_digest_matches.rs:3967`) already runs inside `audit_project`'s read
   scope, so it shares one read per invocation, which is that scope's existing contract for a
   read-only report.
3. `changed_approved_delta_modules` becomes a projection of the memoized binding: modules with
   `intact == false`, or the same `Err`. Its outputs and the refusal text of
   `ensure_approved_delta_bodies_unchanged` stay byte-identical. Existing #704, #719 and #730
   tests pin them.
4. `materialized` is computed from the same body. The per-module loop of
   `prepare_pending_delta_application` is factored into
   `prepare_module_application(root, record, specs_dir, module, delta_body, &mut prepared)`.
   `prepare_pending_delta_application` keeps its read and calls it, and
   `approved_delta_summaries` calls it with `BoundDelta.body`. The canonical `*.spec.md` and
   `requirements.md` are a different input and are read there, as today.

So within one invocation, `summary.changed_approved_deltas`, `summary.approval_valid`,
`delta_intact`, the reported IDs and sections, and `materialized` all come from one read of each
delta file. What this does not cover, stated honestly:

- The canonical spec read behind `materialized` is a separate read from the delta read. A
  concurrent `change check` between them can make `materialized` lag by one invocation. That is
  inherent and the same as reading the tree after the JSON is printed.
- `effective_definition`, `corrections`, `acceptance_entries` and `briefing` read their own files
  today. They do not read delta bodies, and this change does not alter them.
- Outside status, show and mutation results (direct library callers of
  `approved_delta_summaries` with no memo installed), the function installs a memo for its own
  call only. Its output is internally consistent, but a separate `summarize_change` call made by
  the same caller may read the deltas again.

## Domain function (`src/change/collect_requirement_ids.rs`)

`pub fn approved_delta_summaries(root: &Path, record: &ChangeRecord) -> Vec<ApprovedDeltaSummary>`

1. Return `[]` unless `record.state` is Approved, Implementing or Verifying, the record is not
   `no_spec_change`, and `ensure_definition_approval_valid(root, record)` is `Ok`. This is the same
   gate `approval_currency` uses. A stale scope means no current approval covers any delta, and
   `summary.approval_valid` and `scope_expansion` already report it.
2. Take the memoized `approved_delta_binding`.
3. Map to entries:
   - `Err(_)`, meaning a withdrawn binding or an unreadable or unsafe delta: every
     `record.affected_specs` module gets `delta_intact: Some(false)` and nothing else. This matches
     `approval_currency`, which reads the same `Err` as `approval_valid: false`.
   - `Unrecorded`: every module gets `delta_intact: None` and nothing else.
   - `Recorded` with `intact == false`: `Some(false)` and nothing else.
   - `Recorded` with `intact == true`: run `parse_delta(&body)` on the same string.
     - `Ok(items)`: split the items by target and operation into `requirements` and `sections`,
       and set `materialized` as below.
     - `Err(message)`: `delta_intact: Some(true)`, `declarations_error: Some(message)`, no lists,
       no `materialized`. This is not reported as "not intact", because the digest matched.
4. `materialized` for intact, parsed modules:
   - `false` when `!record.canonical_applied`. There has been no materialization, and on a first
     run the application step may raise refusals that only a first application should raise.
   - Otherwise one in-memory `prepare_module_application` call for that module with the bound
     body (no writes): `true` exactly when the module is not pushed to `pending`. `Err` maps that
     module to `false`. Status cannot confirm the tree, and `false` is the conservative answer.

## Consistency invariants

For every fixture, unit tests assert:

1. `delta_intact == Some(false)` if and only if the module is in `summary.changed_approved_deltas`,
   or the binding was refused as a whole (`approval_valid` false and `changed_approved_deltas`
   empty, every entry `Some(false)`).
2. In an active state with a current scope, `summary.approval_valid == true` if and only if no
   entry has `delta_intact == Some(false)`.
3. `requirements`, `sections` and `materialized` are present exactly when `delta_intact == Some(true)`
   and `declarations_error` is absent. `declarations_error` is present only when
   `delta_intact == Some(true)`.

The parse-failure case is in the fixture set (`testing.md`), so invariants 1 and 2 are checked on
it too: `approval_valid` stays true, `changed_approved_deltas` stays empty, and the entry is
`Some(true)` with `declarations_error`.

Static fixtures cannot show a race, so a separate test asserts the mechanism instead: inside a
read scope, `summarize_change_with_strict` followed by `approved_delta_summaries` performs exactly
one binding read per delta file (a `#[cfg(test)]` counter in the style of
`TEST_CANONICAL_MODULE_QUERY_COUNT`), and a delta edited between the two calls is not seen by the
second one.

## Command layer (`src/commands/change.rs`)

- `print_record` adds `"approved_deltas": change::approved_delta_summaries(root, record)`. It runs
  inside the read scope status and show already install, so it shares the memoized binding with
  the `summary` computed a line earlier.
- `print_mutation_record` adds `"approved_deltas": result.approved_deltas`, the value
  `definition_mutation_result` built under the project lock with the same memo as `summary`. It
  computes nothing after persistence, matching the existing comment that the JSON reuses the
  in-lock snapshot. `answer` and `supersede` are draft-only and always produce `[]`. `change new`
  renders through `print_record` instead, because `create_change` returns no mutation snapshot,
  and its `approved_deltas` is always `[]` because a new change is a draft.
  `change depend` is allowed in Approved, Implementing and Verifying
  (`add_dependency_with_snapshot`, `src/change/sdd_version.rs` around line 2552), so its output
  can carry real entries, and it has its own test.

Text paths, `print_roster`, `print_transition*`, `ChangeSummary` and the MCP tools are untouched.
The values are module names, requirement IDs, section names, booleans and a parser diagnostic. No
digest reaches any sink.

## Not reused: `collect_requirement_ids`

The private `collect_requirement_ids` (`src/change/collect_requirement_ids.rs:1557`) also walks
deltas, for verification requirement coverage in `change check`. It reads with unbounded
`fs::read_to_string`, merges added and modified, and drops removed. It stays separate in this
change, on purpose:

- It is a gate input of `change check`, which must read live (no memo), and routing it through the
  bounded binding read would change `change check`'s behavior and refusal text for an oversized or
  unreadable delta. That is a gate change outside this scope (status-only, additive JSON).
- Its question differs. Coverage wants the IDs of the body on disk whether or not it is intact,
  because the binding refusal is reported separately and earlier.

The new code does share the parser (`parse_delta`) and the per-target split, so there are two
readers but one grammar. Moving `collect_requirement_ids` to `read_bounded_change_text` is noted as
a possible follow-up in `tasks.md` for the human to decide; this draft does not file or change it.

## Rejected alternatives

- Placing it in `ChangeSummary`: rejected in `research.md`. It would also reach the roster and
  audit and read canonical specs once per listed change.
- Reporting IDs whenever the body parses, with `delta_intact` beside them: this breaks the
  issue's own rule ("never report a requirement as approved when the delta body changed after
  approval") for any consumer that reads the list and skips the flag.
- Mapping an intact body the grammar refuses to `delta_intact: false`: it would report an approved
  body as edited, contradict `summary.approval_valid` and `changed_approved_deltas`, and break
  invariants 1 and 2.
- Mapping that case to `delta_intact: true` with no lists and no reason: a consumer could not tell
  it from an older binary's omission. `declarations_error` names it.
- Computing `approved_deltas` in `print_mutation_record` after the lock is released: it would mix
  an in-lock `summary` with an out-of-lock read for `change depend` on an approved change.
- Dropping the mutation outputs from scope: `change depend` on an approved change would then lack
  a key that `status` carries for the same record, and a consumer could not rely on "absent means
  older binary".
- Publishing the digest construction so consumers can compare it themselves: this freezes an
  internal that changed in #730. The issue asks for the opposite.
- Recording a `materialized_delta_digests` field: rejected for #741 already. It is a live
  `ChangeRecord` field that moves four definition-digest paths.

## Observable changes and compatibility

| Surface | Before | After |
|---------|--------|-------|
| `change status <id> --json`, `change show <id> --json`, `change new/answer/depend/supersede --json` | no `approved_deltas` key | `approved_deltas` array, always present |
| `summary.*`, `change list --json`, `change status --json` without an ID, `change approve --json` | unchanged | unchanged values; within one invocation `summary` now shares the delta read with `approved_deltas` |
| Text output | unchanged | unchanged |
| Persisted files, digests, ledgers, materialization, gates | unchanged | unchanged; gates read live as before |

The change is additive JSON. A strict-schema consumer that rejects unknown top-level keys would
see a new key. This JSON has no published schema. `acceptance_entries` (#503) and `briefing`
(#804) were added to the same object in the same way. It is still classified as a public-contract
change, because tools such as Chough will depend on the field's meaning.
