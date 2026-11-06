---
id: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
state: draft
type: documentation
base_commit: ad50bdef3db58a715877877d2407485bcaed0f32
---

# Retire the contract clauses that promise change check runs configured verification commands

## Intent

Retire the contract clauses that promise change check runs configured verification commands

## Affected Canonical Specs

- `change`
- `cmd_comment`

## Acceptance Criteria

- SHIP-1.c.1: REQ-change-015 is retired, and REQ-change-019, REQ-change-027, REQ-change-030, REQ-change-061 and REQ-change-069 say no configured verification command is executed and cite REQ-change-023; the change spec, testing.md, context.md and change.3md keep no clause that promises execution.
- SHIP-1.c.1: REQ-cmd-comment-001 and REQ-cmd-comment-002 are retired because REQ-cmd-comment-004 reversed them; REQ-cmd-comment-003, the top-level acceptance list and the spec scenario say comment executes no configured SDD verification command; context.md and cmd_comment.3md no longer say change check streams one.
- SHIP-1.c.1: the why-specsync and adversarial-proof comparison pages no longer credit SpecSync with executing configured tests or isolating a verification shell, and say verification runs no configured command.
- SHIP-1.c.2: change_contracts_give_one_answer_on_configured_verification_commands, comment_contracts_say_no_configured_verification_command_runs and comparison_pages_do_not_credit_specsync_with_running_configured_tests fail on main and pass after the change.
- Runtime behavior is unchanged: no production source changes (only test files under src/change_tests and tests/integration), and the REQ-change-091 discriminators and neither_comment_nor_change_check_spawns_configured_project_commands still pass unmodified; REQ-change-019 and REQ-change-061 gain characterization tests whose mutation checks fail.

## No-spec Rationale

Not applicable
