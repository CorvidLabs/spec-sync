---
change: finalize-archives-only-approvals-that-history-holds
artifact: testing
---

# Testing

src/change_tests/committed_approvals.rs:

- `finalize_refuses_approvals_edited_since_head`: after review, the first approval's actor is
  rewritten in the working tree; finalize refuses naming `approvals.json` and `HEAD`, and the
  change is not archived. Red on main: it archives.
- `finalize_refuses_a_committed_rewrite_of_an_earlier_approval`: the rewrite is committed before
  verification and review; finalize refuses naming that commit. Red on main: it archives.
- `finalize_accepts_a_committed_relay_removal`: a relay is committed, then removed in a later
  commit, as `change repair-relays` does; finalize succeeds. Passes on main and must keep
  passing.
- Existing finalize, ship, archive and reopen tests (unit and integration) stay green; fmt and
  clippy clean.
