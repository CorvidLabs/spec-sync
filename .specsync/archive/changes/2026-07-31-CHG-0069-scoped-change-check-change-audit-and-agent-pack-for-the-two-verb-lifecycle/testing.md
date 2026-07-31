---
change: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
artifact: testing
---

# Testing

## Automated evidence

- `cargo test agents::` — install creates check/audit commands; skill contains Lifecycle verbs
  (REQ-agents-check-audit-commands-001).
- `cargo test commands::change::` / CLI dispatch — Check vs Audit routing
  (REQ-cmd-change-check-scoped-001, REQ-cmd-change-audit-001, REQ-commands-change-audit-dispatch-001,
  REQ-cli-change-audit-001).
- `cargo test change::` — `audit_project` / scoped check_project options
  (REQ-change-audit-project-001, REQ-change-check-scoped-002).
- `cargo test hooks::` — snippet two-verb wording (REQ-hooks-two-verb-001).
- Integration: stale accepted evidence uses `change audit`; scoped check success prints `verified`
  without archive `authenticated-history` lines.

## Manual

- `specsync change audit` on a large-archive repo finishes in seconds.
- `specsync change check <id>` does not print archive evidence dumps.

## Requirement evidence

- `REQ-cmd-agents-0069-001`, `REQ-cli-args-0069-001`: finalization acceptance-manifest construction assigns deterministic owners for `src/commands/agents.rs` (`cmd_agents`) and `src/cli.rs` (`cli_args`) while the check/audit agent pack remains installable.
