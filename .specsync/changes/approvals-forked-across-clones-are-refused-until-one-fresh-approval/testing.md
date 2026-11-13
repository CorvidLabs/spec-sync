---
change: approvals-forked-across-clones-are-refused-until-one-fresh-approval
artifact: testing
---

# Testing

src/change_tests/forked_approvals.rs:

- `approvals_forked_across_clones_are_refused_until_one_fresh_approval`: approvals by clone-a on
  main and clone-b on a branch, merged with `-s ours`; validation refuses naming clone-a,
  clone-b and `specsync change approve <id>`; the summary reports the approval not valid; one
  fresh approval settles it. Red on main: validation passes.
- `approvals_that_only_append_are_not_a_fork`: two committed approvals on one line pass.
- The full unit suite stays green (process-count tests included); fmt and clippy clean.
