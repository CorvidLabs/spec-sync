---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: tasks
---

# Tasks

- [x] Recompute every recorded delta digest across all 198 archived `approvals.json` under both
      the raw and the normalizing preimage, before writing any code, and stop if one moves.
- [x] Add `canonical_delta_body` (CRLF folded to LF, guarded `Cow`, nothing else folded) and frame
      it in `delta_body_digests`.
- [x] Write the discriminator and run it against a binary built from a separate checkout of
      unfixed `main`; record the refusal verbatim.
- [x] Write the controls that fix the boundary — a real wording change in CRLF, four
      applier-equal whitespace edits, the lone carriage return, and the literal pre-change digest —
      and confirm each passes on that same control binary.
- [x] Sweep the other digest sites (`definition_digest`, `definition_artifact_snapshot`,
      `approved_scope`, `project_input_digest`) for the same raw-bytes-versus-normalized mismatch
      and record the result without widening this change.
- [x] Correct the spec wording that said "exact bytes", in `change.spec.md` invariant 38 and
      `REQ-change-089`, and fold the reasoning into `specs/change/context.md`.
- [x] `cargo fmt --check`, CI-equivalent `cargo clippy -- -D warnings`, and the full suite.
