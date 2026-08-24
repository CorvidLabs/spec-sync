---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: plan
---

# Plan

1. Add `APPROVED_DELTA_DIGEST_DOMAIN` and the `ApprovalRecord.approved_delta_digests` field with the
   omitted-when-absent encoding; update every construction site.
2. Add `delta_body_digests` and `ensure_approved_delta_bodies_unchanged`.
3. Record at `append_approval` for the definition gate, and on the acceptance-time normalized
   definition approval.
4. Check in `materialize_change_deltas` (above the `canonical_applied` short-circuit) and in
   `accept_change_with_gate`.
5. Extend the CHG-0068 allowlist shape pin to the new field.
6. Write the discriminator, the control and the compatibility test; prove the discriminator fails
   with the check disabled and that the other two pass with it disabled.
7. Update the canonical spec: correct invariant 3, add invariant 36, add `REQ-change-089`, and
   correct the `ApprovalRecord` Public API description.
8. `cargo fmt`, `cargo clippy -- -D warnings`, full `cargo test`, `change check`,
   `change audit --strict`.
