---
change: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
artifact: context
---

# Context

- Issues #870, #872, #875 and #876 were filed from the #865 review, which added the git merge driver (`src/merge_driver.rs`, `merge_contract`). The fix rounds there stopped after the third round, so these were tracked instead. They are the triage group G-merge-render (`_audits/specsync-triage-2026-11-06.md`).
- In each one the driver reports a clean merge whose result is wrong:
  - #872: a quoted YAML-typed list item is written plain, so YAML reads it as a number, boolean or null.
  - #875: a blank line after the change-log table is dropped, and rows of a following table can be absorbed and renumbered.
  - #876: version comments are taken from `ours` only, so the result depends on merge direction and loses a comment.
  - #870: the change-log `Version` renumbering overflows at `u64::MAX`, panicking in debug and wrapping to 0 in release.
- HI: MERGE-5 and MERGE-5.a to 5.d, drafted from these issues. Leif approved them as drafted on 2026-11-06 ("Capture MERGE-5 + cases"), relayed by the orc (t18). MERGE-1 (either order) and MERGE-4 (no looser gates) also apply.
- REQ-merge-006 is assigned by the orc's registry.
- Red first: the four new tests in `src/merge_driver.rs` fail on main ab688246 for exactly the reasons in their issues. They are committed with this draft.
- Human gates: scope approval and implementation review come from Leif through the orc. This agent never records them as Leif.
