---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: plan
---

# Plan

1. **Measure the archive first.** Recompute every recorded delta digest under both preimages
   across all archived `approvals.json`. If any moves, this is a migration and not a one-line fix,
   and the plan stops here. (It did not move; see `research.md`.)
2. **Build a control.** A separate checkout of unfixed `main` at `d6f266a4`, kept clean, so every
   assertion can be judged against a real binary rather than against the fix disabled in place.
3. **Add the helper and the call site.** `canonical_delta_body` folding CRLF only, framed by
   `delta_body_digests`. The doc comment carries the reasoning and, more importantly, the limit —
   what may not be folded and why.
4. **Write the discriminator, then the controls.** The discriminator alone is satisfiable by
   "normalize everything", so the controls are written in the same pass and each is run on the
   control binary to establish which is which.
5. **Sweep for siblings.** Enumerate every digest site and every reader of change-artifact text,
   determine whether any has the same mismatch, and record the answer either way without widening
   this change.
6. **Correct the spec wording.** "Exact bytes" described something the code should not have been
   acting on; fix invariant 38, `REQ-change-089`, and the `context.md` note that repeats it.
7. **Gates.** `cargo fmt --check`, CI-equivalent `cargo clippy -- -D warnings`, full suite,
   `specsync change check`, `specsync change audit --strict`.
