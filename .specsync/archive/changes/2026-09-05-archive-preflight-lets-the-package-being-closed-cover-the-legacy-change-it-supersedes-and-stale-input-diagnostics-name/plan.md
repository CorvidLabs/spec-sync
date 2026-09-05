---
change: archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name
artifact: plan
---

# Plan

1. Reproduce in `src/change_tests.rs` with the v2 harness (`current_workflow_record`, `check_change`, `record_scoped_review`, `finalize_change`) on a legacy predecessor built by `completed_section_only_record`; confirm the field message on ac796b8.
2. Surface every refusal first, so the reproduction names the failing check; then read the closing package's succession entry from the working tree and forward the token.
3. Re-anchor the stale-successor tests on one fixture, add the authentication-refusal discriminator, and extend the finalize test with the no-reopen guidance tail.
4. Amend REQ-change-020/024/036 and the `Error Cases` table through the delta; record the lesson in `specs/change/context.md`.
5. `fledge run lint`, `fledge lanes run pre-push`, `fledge lanes run verify`; commit; open the PR against `main` citing the swift-algorand reproduction, #751 and #688.
