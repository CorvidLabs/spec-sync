---
change: retire-the-merge-empty-list-and-init-strict-default-contract-clauses
artifact: context
---

# Context

The independent 2026-11-05 audit (finding D12, issue #884) found two current contracts that each give two
answers. The runtime is not in question, and the existing tests already back the intended behavior.

1. Merge. REQ-merge-001, the top-level acceptance list, context.md, merge.3md and the testing.md regression
   matrix say a failed default Git discovery returns no files, an empty list, a safe no-op. merge.spec.md
   invariants 16 and 17, its scenario, REQ-merge-002 and REQ-cmd-merge-002 say `None`, one `Unknown` result,
   exit non-zero. #578 made that change. The matrix still names `git_discovery_failure_is_a_safe_noop`, which
   was renamed to `git_discovery_failure_is_unknown_not_empty`. The result-status list also leaves out
   `Unknown`.
2. Init. REQ-cmd-init-004 AC1 promises Git repositories "normal strict SDD defaults", while REQ-cmd-init-005
   AC4, the spec invariants and v12 say fresh init writes `enabled: false` and
   `require_change_for_meaningful_files: false`. The user story still says init "enables verified SDD", and
   testing.md cites `init_enables_sdd_for_new_projects`, which no function declares. The audit points out
   that `SddPolicy::default()` keeping `true` for omitted fields is a separate path that does not qualify
   the Git-init clause.

Leif approved the intent on 2026-11-05 at 20:24 MST, relayed by orc t18: UPKEEP-3.b, UPKEEP-3.b.1,
SETUP-1.a.1 and SETUP-1.a.2. Decision (e): leave `init --help` unchanged. No runtime change.
