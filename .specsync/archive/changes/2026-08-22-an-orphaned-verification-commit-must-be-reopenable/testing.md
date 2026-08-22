---
change: an-orphaned-verification-commit-must-be-reopenable
artifact: testing
---

# Testing

## Discrimination, against a separate checkout

Per this repo's protocol the baseline is a separate worktree at `3997fc5b`, never a revert in
place. A baseline-compatible variant of the new test (dropping assertions that name post-fix
symbols, which cannot compile pre-fix) fails there with the exact field message:

    thread '...orphaned_reopen_baseline_probe' panicked:
    reopen must fire when the verification commit is unreachable: "accepted change delivery
    inputs are current (exact or successor-covered); reopen is allowed only when delivery
    evidence is stale"

The full test passes on the patched tree.

## New test

`orphaned_verification_commit_reopens_even_though_inputs_are_unchanged` asserts, in order:

1. the manifest-aware live acceptance digest EQUALS the signed one — inputs did not drift
2. `!accepted_evidence_is_anchored(...)`
3. `ensure_closing_approval_valid(...)` errors with "not in current history"
4. `summarize_change(...).next_action` is the reopen command — the verb `status` names must work
5. `reopen_change(...)` succeeds and yields `Verifying`
6. stale and current digests are EQUAL — the equal-digest reopen is real
7. the cause is `VerificationCommitUnanchored`
8. `reopened_change_preserves_sequence_history(...)` accepts it — pins the sibling at :1979

## Vacuity control, added to an existing test

`accepted_evidence_survives_squash_merge_from_nested_project_root` previously asserted
`unwrap_err()` with "delivery inputs are current". That test PINNED THE DEFECT: it constructed
the unanchored state deliberately and asserted the refusal.

It now pins both directions:

- while the remote default still records the squash — evidence anchored, inputs current —
  `reopen` must still REFUSE with "still anchored in current history"
- after `update-ref -d refs/remotes/origin/main`, with byte-identical inputs throughout, `reopen`
  must succeed and record the cause

Without the first half a fix that made `reopen` admit everything would pass just as happily. This
is the control that makes the widening bounded rather than total.

## Security control

`orphaned_reopen_still_refuses_a_tampered_archive` — same orphaned-commit scenario with the
archived `approvals.json` tampered first. Refused:

    accepted change requires exactly one trusted transition matching its state, verification,
    and closing evidence; found 0; archive restored

The anchor axis is not a laundering path.

## Suite

`cargo test`: 2347 unit + 405 integration, 0 failed. `cargo clippy --bins -- -D warnings` clean.
`cargo fmt --check` clean. `specsync check`: 104/104 exports documented, 62 specs passed.

## Not covered

The field reporter's second deadlock — "definition approval is stale" while state is `accepted`,
with `approve` refusing in that state — is NOT addressed here. It may be the approval-identity
axis or a third one. Notably REQ-change-034 already claims "Definition approval can be refreshed
while accepted when the definition digest is stale", so the code may contradict a stated
requirement. A separate reproduction is being cut to settle it.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-017 | The amended refusal criterion is pinned in both directions by `accepted_evidence_survives_squash_merge_from_nested_project_root`: while the remote default still records the squash the evidence is anchored and `reopen` must refuse with "still anchored in current history"; after `update-ref -d refs/remotes/origin/main`, with byte-identical delivery inputs throughout, the same call must succeed. The refusing half is the vacuity control — without it a fix that made reopen admit everything would pass identically |
| REQ-change-018 | `orphaned_verification_commit_reopens_even_though_inputs_are_unchanged` proves unreachability is admissible on its own: the live acceptance digest equals the signed one, `accepted_evidence_is_anchored` is false, `ensure_closing_approval_valid` errors with "not in current history", and `reopen_change` then succeeds recording `VerificationCommitUnanchored`. That the axis is not a laundering path is proven separately by `orphaned_reopen_still_refuses_a_tampered_archive`, where a tampered archived `approvals.json` is refused with "found 0" trusted transitions and the archive is restored. Discrimination measured against a separate worktree at `3997fc5b`, which fails with the exact field message |
| REQ-change-035 | Assertions 6, 7 and 8 of the new test: the reopen records EQUAL stale and current delivery-input digests, carries `stale_evidence_cause: Some(VerificationCommitUnanchored)`, and is then accepted by `reopened_change_preserves_sequence_history`. Without the recorded cause that sibling reads digest equality as proof the reopen was invalid, strips `historical` status, and freezes `change new` project-wide for any member of an acknowledged sequence collision |
