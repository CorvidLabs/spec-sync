---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: design
---

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
