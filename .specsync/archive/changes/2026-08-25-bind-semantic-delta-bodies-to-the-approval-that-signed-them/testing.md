---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: testing
---

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
