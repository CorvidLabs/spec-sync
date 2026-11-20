---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-169 | `src/change_tests/vanished_requirements.rs`: the starling truncation refused at check naming every lost id (red on main 3bebe8b0), accounted removals pass (same change, another active change, an archived change, merged from main), a cherry-picked main removal without the merge is refused, a moved block passes, a renumber is refused with the permanence hint, a deleted file and a truncated legacy `<module>.req.md` are refused, the `base_commit` fallback, the skip with a note, an unreadable merge-base refuses, and audit reports it |
| REQ-cmd-change-042 | an integration test: `change ship-status` lists the blocker and exits 1, `change ship --dry-run` and `change finalize` refuse, and restoring the blocks clears them |

## How it runs

`cargo test --bin specsync vanished` and the integration test locally (red, then green), and the full verify lane.
