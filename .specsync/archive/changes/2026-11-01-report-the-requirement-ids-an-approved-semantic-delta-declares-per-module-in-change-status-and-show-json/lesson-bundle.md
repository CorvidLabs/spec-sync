# Lesson bundle: report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Report the requirement IDs an approved semantic delta declares, per module, in change status and show JSON
- **Kind**: Feature
- **Specs**: change, cmd_change
- **Paths**: src/change/collect_requirement_ids.rs, src/change/sdd_version.rs, src/change_tests.rs, src/change_tests/approved_delta_requirements.rs, src/commands/change.rs, tests/integration/change.rs, site/src/content/docs/cli.md, specs/change, specs/cmd_change
- **Acceptance**: After a workflow-v2 change is approved, `specsync change status <id> --json` and `specsync change show <id> --json` carry a top-level `approved_deltas` array with one entry per affected module; for the issue #826 reproduction the `shipping` entry reports `delta_intact: true`, `requirements.added: ["REQ-shipping-002"]`, `sections.modified: ["Public API"]` and `materialized: false`.
- **Acceptance**: Editing an approved delta body after approval makes that module's entry report `delta_intact: false` with no `requirements`, `sections` or `materialized`, so no requirement is reported as approved from an unapproved body; the module is the one `summary.changed_approved_deltas` names, `summary.approval_valid` is false and `summary.next_action` names re-approval, as on origin/main a80748f.
- **Acceptance**: Converting an approved delta to CRLF line endings without other edits keeps `delta_intact: true`; restoring or re-approving an edited delta returns it to true; a later definition approval that withdraws a recorded delta binding reports every module `delta_intact: false`; a ledger that never recorded delta digests reports `delta_intact: null` and no requirements, never false.
- **Acceptance**: After a passing `change check` the entry reports `materialized: true`; after a corrected delta is re-approved following materialization it reports `materialized: false` until the next `change check` applies it.
- **Acceptance**: Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` item headings are reported; a requirement ID mentioned in prose inside a delta body is not.
- **Acceptance**: `approved_deltas` is `[]` for draft, accepted and archived changes and for an approved change whose scope approval is stale; it is always present in single-record change JSON; no existing JSON key, `summary` field, `change list --json` output, text output, persisted ledger, digest or materialization behavior changes.
- **Acceptance**: Regression tests fail on origin/main a80748f and pass with the change: unit discriminators and labelled controls for each case above, and an integration test that drives `change status --json` and `change show --json` through the built binary.

## Evidence

- Verification commit: `850f1be1245831570fffed52304061279f5d7a5b`
- Base commit: `a80748ff44ff42cd7578ac0f7cb8ba7e080ab183`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Requested in CorvidLabs/spec-sync#826, reported downstream as CorvidLabs/chough#11. Chough's
`cases check` refuses a test case that links `REQ-shipping-002` while that ID exists only in an
approved semantic delta. Nothing in native SpecSync JSON names that ID until `change check`
materializes it into `specs/<module>/requirements.md`. The only way to find it is to parse
`.specsync/changes/<id>/deltas/*.md`. That duplicates SpecSync's delta grammar. The parser also
cannot tell whether the body on disk is the approved one, because `approved_delta_digests` is a
domain-separated, framed SHA-256 over the module name and the CRLF-folded body
(`delta_body_digests` and `canonical_delta_body` in `src/change/collect_requirement_ids.rs`).
That construction is internal and undocumented, and it changed in #730.

## Reproduction (origin/main a80748f, built from this worktree)

The issue's `base.sh` and `repro2.sh` were run unchanged against a release build of a80748f. The
fixture was Chough's `examples/shipping` at CorvidLabs/chough@2756dc2, and the approver was a
simulated actor in a disposable repository under the session scratchpad. No real approval was
recorded.

A. After approval, REQ-shipping-002 appears in no native output:

```text
{ "state": "approved", "approval_valid": true }
status --json bytes: 2589, mentions: 0
show --json bytes:   2589, mentions: 0
specs/shipping/requirements.md mentions: 0
delta file mentions: 1
top-level keys: ["acceptance_entries","briefing","change","corrections","effective_definition","questions","summary"]
approvals.json approved_delta_digests: [{"shipping":"de755d1e24cbb98b22b938f6efea918cc9a21dd4b08a7a7746173d55673cbe79"}]
sha256 of delta file: 9b610ff6932c9d13de80a2442f8ce7f0d0800762a12eb5fa65dfa5013d3d6fc4
```

B. After the approved body is edited, status is already correct on main. #823 merged
REQ-change-111, which made `approval_valid` false and `next_action` name re-approval:

```text
{"state":"approved","approval_valid":false,"next_action":"run `specsync change approve free-shipping-at-the-threshold --actor <name>`"}
summary.changed_approved_deltas: ["shipping"], handoff: not-yet
```

C. `change check` exits 1 with "semantic delta for `shipping` changed after approval; ...".

With `base.sh <dir> verify`, which approves, implements and runs `change check --commit`, the change
is `verifying` with `canonical_applied: true`. REQ-shipping-002 is now in
`specs/shipping/requirements.md`, and the status JSON still has no field that names it.

So the second half of #826 (an integrity flag in `approval_valid` and `next_action`) shipped in
#823. What is still missing is the first half: a per-module report of the declared requirement IDs
and sections, a per-module integrity flag, and a materialization flag.

## Cause

- `print_record` and `print_mutation_record` (`src/commands/change.rs`) emit `change`,
  `effective_definition`, `acceptance_entries`, `corrections`, `summary`, `questions` and
  `briefing`. None of them carries delta items.
- The only delta-item reader that reports IDs is the private `collect_requirement_ids`
  (`src/change/collect_requirement_ids.rs:1557`). It feeds verification's requirement coverage, it
  does not split added, modified and removed, and it reads with unbounded `fs::read_to_string`.
- The per-module drift answer exists as the private `changed_approved_delta_modules` (`:1365`).
  Status uses it only through `approval_currency` (`:1416`), which reduces it to the
  `changed_approved_deltas` list.
- Whether the canonical tree carries the delta on disk is derived only inside
  `prepare_pending_delta_application` (`:1834`, #741). `canonical_applied` records only that
  materialization ran.

## Constraints already settled

- An absent delta binding is unknown, not a violation. That is judged per ledger: no definition
  approval recorded one (REQ-change-089, REQ-change-090).
- `approval_currency` decides which states the binding is judged in (REQ-change-111). This change
  must read the same predicate and add no second comparison.
- No new live field on `ChangeRecord`. It is hashed into `definition_digest` in four places
  (`change.3md`, #741 plane).
- HI: links CHANGE-5 (the delta is readable: added, changed, retired), CHANGE-6.a (the approval is
  bound to its content, so later edits cannot ride on it), CHANGE-7 (ask the state of a change) and
  AGENT-2 (structured data, so an agent never scrapes). No new HI criterion is proposed or captured.

## Re-run of the issue reproduction (candidate build)

The issue's `repro2.sh` and `base.sh` were run unchanged against a debug build of this branch, with
the same fixture (Chough `examples/shipping` at CorvidLabs/chough@2756dc2) and a simulated
approver in a disposable repository under the session scratchpad.

A. After approval, `status.json` and `show.json` each mention REQ-shipping-002 once, under the new
key:

```text
{ "state": "approved", "approval_valid": true }
status --json bytes: 2979, mentions: 1
show --json bytes:   2979, mentions: 1
top-level keys: ["acceptance_entries","approved_deltas","briefing","change","corrections","effective_definition","questions","summary"]
approved_deltas: [{"delta_intact":true,"materialized":false,"module":"shipping","requirements":{"added":["REQ-shipping-002"],"modified":[],"removed":[]},"sections":{"added":[],"modified":["Public API"],"removed":[]}}]
```

B. After the approved body is edited: `{"approval_valid":false,"next_action":"run `specsync change approve free-shipping-at-the-threshold --actor <name>`"}`
and `approved_deltas: [{"delta_intact":false,"module":"shipping"}]`.

C. `change check` still exits 1 with the unchanged "semantic delta for `shipping` changed after
approval" refusal.

With `base.sh <dir> verify` the change is `verifying` and the entry reports `materialized: true`
with the same requirement and section lists.

## Deviation found while testing

The design expected `change depend` on an approved change to carry real entries. Dependencies
are part of the approved scope, so a NEW dependency stales the scope and the in-lock
`approved_deltas` is `[]`, exactly what `status` then reports. Re-declaring a dependency the
approval already covers keeps the scope current and carries the intact entries. The unit and
integration tests drive both paths.

## Observed results of the regression lanes

Recorded after the review pass on PR #828. Figures marked "first run" come from the first
implementation pass, whose logs were not committed; this pass did not re-run the a80748f stub
labelling. Every other figure was observed in this pass on the working tree that the review fix
commit records.

- Unit, first run: against the a80748f stub described in `testing.md`, 12 failed and 14 passed.
  With the change, 26 passed.
- Unit, this pass: the 17 tests in `src/change_tests/approved_delta_requirements.rs` pass.
  `cargo test --bin specsync change::` gave 557 passed, 0 failed. `cargo test --bin specsync
  commands::change::` gave 34 passed, 0 failed.
- Integration, before: `status_and_show_json_report_approved_delta_requirements`, run with
  `CARGO_BIN_EXE_specsync` pointing at a debug build of unfixed a80748f, fails at its first
  assertion. The draft `change status --json` has no `approved_deltas` key (`left: Null`,
  `right: Array []`).
- Integration, after: the same test passes against this branch.
- `summary` key set: the integration fixture was replayed through both binaries. In the draft,
  approved and `change new --json` outputs, a80748f and this branch print the same 13 `summary`
  keys. The test now asserts that set (testing.md step 2), and asserts that `change new --json`
  carries `approved_deltas == []`.
- Full suite inside `fledge lanes run verify`: 2640 unit tests passed and 465 integration tests
  passed, 0 failed. The lane passed (exit 0), including fmt, clippy, the release build and strict
  specs.
- `specsync check --strict --require-coverage 100`: 62 specs checked, 62 passed, 0 warnings. File
  coverage 130/130 and LOC coverage 147486/147486.
- `specsync change audit`, run before this commit's `change check --commit`, reported only
  "verification evidence is stale for the current commit or contract". That is expected after
  an edit, and the re-run after `change check --commit` is recorded in the PR.
- `fledge lanes run pre-push` and `fledge trust verify` run before each push. Their results for
  each head are recorded in the PR, not here, because a new commit outdates them.

## Review finding: `change new` is not built from a mutation snapshot

REQ-cmd-change-023 AC2 and the first `cli.md` text list `change new` among the outputs built
under the project lock. In the code, `change new` renders through `print_record` after
`create_change` releases the lock. Its `approved_deltas` is always `[]` because a new change is
a draft, so nothing observable differs. `cli.md` and `docs.md` now claim the in-transaction build
only for `answer`, `depend` and `supersede`. The requirement wording then still listed
`change new`. Correcting it was a delta edit that needed re-approval. That could be folded into the REQ-change-114
renumber re-approval that the #829-first merge order already requires. Routing `change new`
through a snapshot would instead add a crate-private function to the approved Public API section,
which also needs re-approval. The human chose between the two (below).

Decision (0xLeif, 2026-11-01): amend, then accept. REQ-cmd-change-023 AC2 in
`deltas/cmd_change.md` no longer lists `change new` among the outputs built in the same
transaction as their `summary`, and states that `change new --json` always carries an empty
`approved_deltas`. Requirement IDs are unchanged. 0xLeif re-approved the amended definition on
2026-11-01, and `change check` re-materialized `specs/cmd_change/requirements.md` to match.

## From the change's design.md

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

## From the change's testing.md

# Testing

Each test carries an honest label. A DISCRIMINATOR fails on origin/main a80748f and passes with
the change. A CONTROL passes on both, or pins behavior that must not move. Labels are confirmed by
running the new tests against a build of unfixed a80748f from a separate checkout, not by
disabling the change in place.

On a80748f no `approved_delta_summaries` exists, so every unit test that calls it fails to compile
there. To label them, apply the tests together with a stub
`approved_delta_summaries(..) -> Vec::new()`, the two types, an always-empty
`DefinitionMutationResult.approved_deltas`, `delta_body_digest` extracted verbatim from
`delta_body_digests` (no behavior change), and a binding-read counter that counts the existing
`delta_body_digests` reads, and nothing else.

## Unit tests (new `src/change_tests/approved_delta_requirements.rs`)

Fixture: `change_with_an_approved_requirement_delta(root)`, built on
`completed_section_only_current_record` with an `auth` delta that has three items:

- `## ADDED` / `### REQUIREMENT REQ-auth-002`, which has SHALL and Acceptance Criteria and whose
  body mentions `REQ-auth-009` in prose,
- `## MODIFIED` / `### SPEC SECTION Purpose`,
- `## REMOVED` / `### SPEC SECTION Legacy Notes`.

The fixture writes requirement evidence for REQ-auth-002 into `testing.md`, then
`approve_definition`.

Labels follow the definition above, applied to the a80748f stub (`approved_delta_summaries`
returns `Vec::new()`, and `DefinitionMutationResult` has an empty `approved_deltas`). Any test that
expects a non-empty entry fails against that stub, so it is a DISCRIMINATOR, including tests whose
subject (CRLF, restore) is behavior main already has in `summary`. Only tests that expect `[]` or
pin pre-existing behavior are CONTROLS.

DISCRIMINATORS:

- `approved_delta_summaries_report_declared_requirements_and_sections`: approved state. The
  `auth` entry has `delta_intact == Some(true)`, `requirements.added == ["REQ-auth-002"]`,
  empty `modified` and `removed` requirements, `sections.modified == ["Purpose"]`,
  `sections.removed == ["Legacy Notes"]`, `materialized == Some(false)` and no
  `declarations_error`.
- `a_requirement_mentioned_in_prose_is_not_reported`: `REQ-auth-009` is in no list.
- `an_edited_approved_delta_reports_no_approved_requirements`: body edited after approval.
  - The entry has `delta_intact == Some(false)` and `requirements`, `sections`, `materialized`
    and `declarations_error` all `None`.
  - The serialized entry has none of those keys.
  - `summarize_change` gives `changed_approved_deltas == ["auth"]` and `!approval_valid`.
- `a_withdrawn_delta_binding_reports_every_module_not_intact`: uses the ledger shape of
  `a_later_definition_approval_may_not_withdraw_a_recorded_delta_binding`.
- `a_ledger_without_delta_digests_reports_unknown_integrity`: `strip_recorded_delta_digests`.
  - `delta_intact == None`, serialized as `null`, with no lists.
  - It is not `Some(false)`, even after the body is edited.
- `an_intact_delta_the_current_grammar_refuses_reports_a_declarations_error`: approve the
  fixture, overwrite the `auth` delta with a body `parse_delta` refuses (an `## RENAMED`
  operation heading), and rewrite the effective approval's `approved_delta_digests["auth"]` to
  `delta_body_digest("auth", body)`. That is the ledger an approval under a more permissive
  grammar would have left. Expects `delta_intact == Some(true)`, a `declarations_error` naming
  `## RENAMED`, and no `requirements`, `sections` or `materialized`. `summarize_change` keeps
  `approval_valid == true` and `changed_approved_deltas == []`.
- `materialized_turns_true_after_check_and_false_after_a_reapproved_correction`:
  - After `materialize_change_deltas`, `materialized == Some(true)`.
  - On `change_materialized_then_corrected_and_re_approved`, `materialized == Some(false)` and
    the `requirements` and `sections` are those of the corrected body.
  - After a second `materialize_change_deltas`, it is `Some(true)` again.
- `approved_delta_summaries_hold_in_implementing_and_verifying`: the same entry after
  `start_implementation`, and after a passing `check_change`.
- `a_crlf_checkout_of_an_approved_delta_stays_intact`: uses
  `approved_delta_body_as_a_crlf_checkout_writes_it`. Expects `Some(true)` and the same lists.
- `restoring_or_reapproving_an_edited_delta_restores_intact`: both paths return `Some(true)` with
  the lists of the restored or re-approved body.
- `status_projections_share_one_delta_binding_read`: inside `begin_change_read_scope`, call
  `summarize_change_with_strict` (normal and strict) and then `approved_delta_summaries`. The
  `#[cfg(test)]` binding-read counter shows one read per affected delta. Then edit the delta and
  call `approved_delta_summaries` again in the same scope: it still reports the memoized intact
  entry, agreeing with the summary already built. In a fresh scope it reports `Some(false)`. On
  a80748f the stub returns `[]`, so the agreement assertion fails.
- `a_dependency_added_to_an_approved_change_reports_its_approved_deltas`:
  `add_dependency_with_snapshot` on an approved fixture. `result.approved_deltas` has the intact
  `auth` entry, and invariants 1 to 3 hold between it and `result.summary`.

CONTROLS:

- `approved_delta_summaries_are_empty_outside_active_approved_states`: draft, a stale-scope
  approved change (`affected_paths` edited), accepted, archived and `no_spec_change` changes all
  return `[]`.
- `definition_mutations_on_drafts_report_no_approved_deltas`: `new`, `answer` and `supersede`
  results carry `approved_deltas == []`.
- `approved_delta_summaries_agree_with_changed_approved_deltas`: for every fixture above,
  including the `declarations_error` one, assert invariants 1 to 3 in `design.md`. Against the
  stub, the empty list makes the invariants vacuously true, so this is a CONTROL; it guards the
  candidate, not the bug.
- `gates_read_delta_bodies_live_outside_a_memo`: with no memo installed, edit the delta between
  two `ensure_approved_delta_bodies_unchanged` calls. The second call refuses. This pins that the
  memo never reaches materialization or acceptance.
- `delta_binding_refactor_keeps_existing_refusals`: `ensure_approved_delta_bodies_unchanged` and
  `changed_approved_delta_modules` give the same `Ok`, `Err` strings and lists as before. The
  existing #704, #719 and #730 tests and `status_reports_an_edited_approved_delta_as_an_invalid_approval`
  stay green unchanged.
- `prepare_pending_delta_application` refactor: the existing #741 and registry-path tests in
  `src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs` stay green
  unchanged.

## Integration test (`tests/integration/change.rs`)

`status_and_show_json_report_approved_delta_requirements` runs through the built binary under a
workflow-v2 `sdd.json`, following `status_json_and_text_report_an_edited_approved_delta`:

1. Create, answer, complete and approve a change with a fixture actor. Its delta adds
   `REQ-auth-002` and modifies one section.
2. Both `change status <id> --json` and `change show <id> --json` have `approved_deltas[0]` with
   `module == "auth"`, `delta_intact == true`, `requirements.added == ["REQ-auth-002"]` and
   `materialized == false`. The other top-level keys and `summary` keys equal the set on a80748f.
3. Re-encode the delta to CRLF. Still `true`.
4. Edit the delta body: `delta_intact == false`, no `requirements` key,
   `summary.changed_approved_deltas == ["auth"]`. Restore it.
5. Run `change check`. `materialized == true`.
6. Text `change status <id>` output is unchanged and contains no `REQ-auth-002`.
   `change list --json` has no `approved_deltas`.
7. A draft change's `change status --json` has `approved_deltas == []`.
8. `change depend <id> <other> --json` on the approved change has the same `approved_deltas`
   entry as `change status <id> --json`.

## Re-run of the issue reproduction

Run the issue's `repro2.sh` against the built candidate. Section A should find
`REQ-shipping-002` in `status.json` and `show.json` under `approved_deltas`, and section B should
show `delta_intact: false`. Record the output in `context.md`.

## Regression lanes

- `cargo test change::` (the component route for `change`).
- `cargo test commands::change::`.
- `cargo test --test integration change::`.
- `fledge lanes run verify` (fmt, clippy, types, full test, release build, strict specs).
- `specsync check --strict --require-coverage 100`.
- `specsync change audit`.
- `fledge lanes run pre-push` before every push.
- `fledge trust verify` before calling the work complete.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-change-113 | Discriminators: `approved_delta_summaries_report_declared_requirements_and_sections`, `a_requirement_mentioned_in_prose_is_not_reported`, `an_edited_approved_delta_reports_no_approved_requirements`, `a_withdrawn_delta_binding_reports_every_module_not_intact`, `a_ledger_without_delta_digests_reports_unknown_integrity`, `an_intact_delta_the_current_grammar_refuses_reports_a_declarations_error`, `materialized_turns_true_after_check_and_false_after_a_reapproved_correction`, `approved_delta_summaries_hold_in_implementing_and_verifying`, `a_crlf_checkout_of_an_approved_delta_stays_intact`, `restoring_or_reapproving_an_edited_delta_restores_intact`, `status_projections_share_one_delta_binding_read`, `a_dependency_added_to_an_approved_change_reports_its_approved_deltas`. Controls: `approved_delta_summaries_are_empty_outside_active_approved_states`, `definition_mutations_on_drafts_report_no_approved_deltas`, `approved_delta_summaries_agree_with_changed_approved_deltas`, `gates_read_delta_bodies_live_outside_a_memo`, `delta_binding_refactor_keeps_existing_refusals`. |
| REQ-cmd-change-023 | Integration: `status_and_show_json_report_approved_delta_requirements`. Existing REQ-cmd-change-005 coverage, including `status_json_and_text_report_an_edited_approved_delta`, keeps passing. |

No result is claimed here. Observed results are recorded in `context.md`, "Observed results of the regression lanes".

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
