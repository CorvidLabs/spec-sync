---
change: refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it
artifact: context
---

# Context

Audit D1 (spec-sync#694, #690 related): after a squash before finalize, `change review` passed and recorded acceptance with handoff `safe`, then `change finalize` failed with "requires exactly one trusted transition matching its state, verification, and closing evidence; found 0". After that `check --commit` refused (accepted), `reopen` refused (evidence current), and `status` still said `safe` and pointed at finalize.

Instrumenting finalize on the auditor's fixture (main 884a33d3) showed the squash is not the cause. `archive_introduction_index` and `accepted_transition_anchors` walk `HEAD` and the remote default ref. The fixture's `origin/main` already holds `chore(lifecycle): archive greeting-version-1` from the original timeline, so `archived_package_is_in_history` is true and the working tree's closing evidence may not speak. The identical squash with `git remote remove origin` reviews and finalizes normally.

Refusing to finalize a change that is already archived upstream is correct. This change makes the refusal early and named. Leif chose this scope on 2026-11-05 18:53 ("A': named refusal"), replacing an earlier A+B choice whose reopen half would have produced a duplicate archive. The root cause is recorded on #694.

Out of scope: #694's scoped-review descendant walk (its options 1-3) and any reopen path.
