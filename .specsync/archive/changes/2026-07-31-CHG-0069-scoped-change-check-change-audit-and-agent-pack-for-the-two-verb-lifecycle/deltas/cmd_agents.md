## ADDED

### REQUIREMENT REQ-cmd-agents-0069-001
The agents command module SHALL retain deterministic canonical ownership of `src/commands/agents.rs` while shipping `/specsync:check` and `/specsync:audit` install targets.

Acceptance Criteria
- `src/commands/agents.rs` has deterministic canonical ownership via `cmd_agents`.
- Agent pack install continues to dispatch through the agents command entrypoint.
