---
change: retire-the-merge-empty-list-and-init-strict-default-contract-clauses
artifact: design
---

# Design

| Clause | Was | Becomes |
|---|---|---|
| REQ-merge-001 AC (and the top-level copy) | returns no files, a safe no-op | returns `None`; one `Unknown` result naming the precondition and `--all`; nothing written; exits non-zero (REQ-cmd-merge-002) |
| REQ-merge-001 results AC (and the top-level copy) | `Resolved` / `Manual` / `Clean` | adds `Unknown` |
| merge context.md, merge.3md | returns an empty list | returns `None`; one `Unknown` result (#578) |
| merge testing.md row | empty list; `git_discovery_failure_is_a_safe_noop` | `None` and `Unknown`; the three tests that pin it |
| REQ-cmd-init-004 | SHALL enable Git-dependent coverage; Git gets strict defaults | the same SDD-off values in Git and non-Git directories; `SddPolicy::default()` only fills omitted fields |
| cmd_init user story | init enables verified SDD and detects test commands | layout and source dirs, change workflow off until `change adopt` |
| cmd_init testing.md | cites `init_enables_sdd_for_new_projects` | cites the real tests, plus a REQ-cmd-init-004 evidence line |

Requirement blocks change through the semantic deltas. The top-level lists, companions and lessons are
direct edits inside the declared paths. Historical change-log rows stay as they are.
