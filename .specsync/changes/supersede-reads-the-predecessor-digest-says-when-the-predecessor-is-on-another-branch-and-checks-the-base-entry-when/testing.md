---
change: supersede-reads-the-predecessor-digest-says-when-the-predecessor-is-on-another-branch-and-checks-the-base-entry-when
artifact: testing
---

# Testing

src/change_tests/easier_supersede.rs:

- `supersede_works_out_the_predecessor_entry_digest`: no digest given; the obligation holds the
  predecessor's signed entry digest, and no base warning.
- `a_predecessor_missing_from_this_branch_is_named_as_such`: an unknown predecessor is refused as
  not in this branch's history, with rebase.
- `a_predecessor_entry_moved_before_the_base_is_named_at_supersede_time`: `src/auth.rs` committed
  after acceptance; the obligation is recorded and the warning names the path, the moving commit
  and reconcile-history.
- Existing supersede, succession, reconciliation and documented-workflow tests stay green; fmt
  and clippy clean.
