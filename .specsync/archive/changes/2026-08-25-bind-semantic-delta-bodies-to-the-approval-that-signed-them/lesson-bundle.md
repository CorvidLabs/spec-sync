# Lesson bundle — bind-semantic-delta-bodies-to-the-approval-that-signed-them

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Bind semantic delta bodies to the approval that signed them
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs
- **Acceptance**: A semantic delta body swapped after approval is refused before it rewrites the canonical spec, and the refusal names the module
- **Acceptance**: An untouched approved delta still materializes into the canonical spec exactly as before
- **Acceptance**: An approval that records no delta digest — every archived change in this repository — proceeds, because absent evidence is unknown and not a violation

## Evidence

- Verification commit: `6479b5c18a72e2fbe3433cea71d5c9f17d1cdebb`
- Base commit: `875752ee991d458db172dec6ceb712462fe2a614`
- Verified by: `cargo test change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

#704 demonstrated, end to end, that a semantic delta body can be swapped between `approve` and
materialization and that the swapped wording lands in the living spec with no error and no warning.
The approval ledger goes on asserting that a human signed a definition; the canonical spec carries
text that definition never covered.

Nothing covered the region, and each mechanism misses it for its own reason:

- `definition_digest` under workflow v2 is `scope_digest`, a projection of intent and boundary. It
  deliberately excludes wording so that editing a delta stales verification instead of demanding a
  fresh human approval — but nothing was left holding the wording.
- `validate_delta_files` reads `entry.file_name()`. It checks the module set, never a byte.
- `project_input_digest` excludes `.specsync/changes/` by design (`project_input_is_volatile`).
- the descendant walk would notice, and passes 0 of 107 archived reviews (#694).

Workflow v1 did NOT have this hole: `definition_artifact_snapshot` hashes each delta file's payload
into the v1 definition digest. The binding existed and was dropped at the v2 boundary. Spec
invariant 3 still claimed it, which is why the gap read as covered.

The threat model is the one #704 states, not a larger one: this needs local write access to the
workspace between approve and materialize. It is not remote. What it breaks is evidence integrity —
and the same window is reached without malice by a bad merge, a rebase that resurrects an older
delta, an agent editing the wrong file, or two changes racing on one workspace.

The hard constraint is compatibility. 183 archived changes carry no such digest and never could. An
absent digest must read as "this approval made no claim about wording", never as "the wording was
tampered with". This repository has shipped the opposite reading three times — #672 read an
unparseable schema as every table missing, #684 read a missing config as a gating warning, and
#689's first design would have reported "ready" from absent evidence — so the absent case is the
part of this change that got the most care, and it has its own test.

Ruled out along the way:

- Adding the digests to `ApprovedScopeV1`. That struct is the `scope_digest` preimage; a new field
  changes every existing scope digest and invalidates every live approval.
- Adding a field to `ChangeRecord`. The workflow-v1 definition digest serializes the whole record,
  so a field there is only digest-safe while it is omitted — a trap for the next person.
- Putting the check inside `prepare_delta_application`. It is the one choke point both application
  paths share, but four existing tests call it directly on draft fixtures with no definition
  approval at all, so the check would have had to treat "no approval" as "proceed" — a second
  absent-evidence rule with a much weaker justification than the one this change is built on.

## From the change's design.md

# Design

**Where the evidence lives.** `ApprovalRecord.approved_delta_digests: Option<BTreeMap<String,
String>>`, alongside the existing optional evidence fields, with `#[serde(default,
skip_serializing_if = "Option::is_none")]`. The approval event is the right home because the
question is "what did this approver sign", and because `ApprovalRecord` is a tolerant evidence
struct rather than a digest preimage — see research.md.

No new public type is introduced: a `BTreeMap<String, String>` carries module to digest, and the
version lives where every other digest version in this module lives, in the domain string
`specsync.approved-delta.v1`.

**What is hashed.** `delta_body_digests` frames the module name and the file's exact bytes into a
`FramedDigest` per module. Framing the module means a body moved from `deltas/a.md` to `deltas/b.md`
cannot keep its digest. Keying by module means a refusal can name the file a human has to look at,
which a single whole-directory digest could not.

A `no_spec_change` record yields an empty map rather than `None`. That is the truthful reading:
`validate_delta_files` already refuses any delta file at all for such a change, so the approval
covered zero bodies — a claim, not an absence.

**Where it is checked.** `ensure_approved_delta_bodies_unchanged` resolves the effective definition
approval, and returns `Ok(())` immediately when it carries no digests. It is called from
`materialize_change_deltas` — above the `canonical_applied` short-circuit — and from
`accept_change_with_gate`, the two paths that reach `prepare_delta_application`.

**Which gates record.** Only definition gates. `append_approval` records for `gate == "definition"`
and nothing else, because a closing or finalization approval reviews delivery evidence and claiming
in the ledger that it reviewed wording would be false. The "normalized compatible definition
evidence" approval appended during explicit acceptance is a definition gate, so it carries the
binding forward rather than dropping it; the bodies it names were checked earlier in the same call,
so it can only record verified wording.

**What is deliberately not covered.** `append_portable_definition_approval_v501` records nothing. It
refuses any record that is not workflow v1, and the v1 definition digest already hashes delta
payloads through `definition_artifact_snapshot`, so the binding is present there by another route.

**The allowlist.** `validate_scope_adoption` pins the entire field shape of the one trusted CHG-0068
event; the new field is added to that pin so the allowlist stays exhaustive over the struct.

## From the change's testing.md

# Testing

Three tests in `src/change_tests.rs`, each labelled for what it actually discriminates.

**`a_semantic_delta_swapped_after_approval_never_reaches_the_canonical_spec` — DISCRIMINATOR.**
Approves the `auth` delta, overwrites the file with unapproved wording, calls
`materialize_change_deltas`. Asserts the refusal names `` `auth` `` and says "changed after
approval", that `specs/auth/auth.spec.md` does not contain the swapped text, and that the record did
not mark itself applied. Verified to FAIL with the check disabled: the unfixed path returns
`Ok(ChangeRecord { canonical_applied: true, .. })` and the backdoor text lands in the spec.

**`an_approved_delta_that_was_never_touched_still_rewrites_the_canonical_spec` — CONTROL.**
Honest label: this passes on the unfixed binary too, and that is the point. It fails only if the new
check starts refusing honest work, which would be an outage rather than a fix. It also pins the
positive half — the approval carries a digest keyed by `auth` — so "the check passed" cannot quietly
mean "there was nothing recorded to check". Verified to PASS with the check disabled.

**`an_approval_recorded_before_delta_digests_existed_is_unknown_not_violated` — COMPATIBILITY.**
Strips the field from the ledger so the file is shaped exactly like a pre-#704 one (asserted on the
raw bytes, not assumed), swaps the delta anyway, and requires materialization to proceed. It fails
the moment someone decides a missing digest should read as tampering — which would fail all 183
archived changes on evidence nobody could have written. Verified to PASS with the check disabled.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-089 | `a_semantic_delta_swapped_after_approval_never_reaches_the_canonical_spec` (discriminator: refusal names `` `auth` `` and says "changed after approval", spec untouched, record not marked applied — verified to FAIL with the check disabled); `an_approved_delta_that_was_never_touched_still_rewrites_the_canonical_spec` (control: untouched delta still materializes and the approval carries a per-module digest — passes on both binaries, honestly labelled); `an_approval_recorded_before_delta_digests_existed_is_unknown_not_violated` (compatibility: field stripped so the ledger is byte-shaped like a pre-binding one, delta swapped anyway, materialization must proceed); plus the full `cargo test --bin specsync` suite as the regression net for the field addition |

Beyond the three: the full `cargo test --bin specsync` suite passes, which is the regression net for
the field addition across every approval, reopen, correction and archive path.

Not covered by a unit test: the `accept_change_with_gate` call site. It is the same
`ensure_approved_delta_bodies_unchanged` call as the materialization path, placed next to the same
`validate_delta_files`, but reaching it from a test needs a full verify/accept fixture and the
existing suite's accept paths all run with untouched deltas. Stated here rather than implied.

## Where these lessons go

- `specs/change/context.md`
