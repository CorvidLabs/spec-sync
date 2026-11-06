---
change: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
artifact: context
---

# Context

The independent 2026-11-05 audit (finding D4, issue #848) found that the current SpecSync contracts
give two answers to one question. `specs/change/requirements.md` REQ-change-015 says reporting mode
"still executes every configured verification command", while REQ-change-023, REQ-change-049,
REQ-change-058 and REQ-change-091 say `change check` executes none, and the runtime agrees with the
latter. A deliberately wrong greeting passed `change check` and failed only the project's own test.
The audit asked for one current normative answer, and warned against fixing it by quietly adding test
execution to a structural command.

A sweep for the same promise found more clauses: REQ-change-019, REQ-change-027, REQ-change-030,
REQ-change-061 and REQ-change-069 in the change module; the top-level acceptance list,
REQ-cmd-comment-003 and one behavioral scenario in cmd_comment, plus a status line saying
`change check` streams the configured output; and three rows on two public comparison pages.
REQ-cmd-comment-001 and REQ-cmd-comment-002 contradict REQ-cmd-comment-004 for the same root cause:
#543 cut `comment` off from the lifecycle path that ran these commands.

Leif's decisions (2026-11-05 20:24 MST, relayed by orc t18): the intent was approved as drafted
(SHIP-1.c.1, SHIP-1.c.2). On a to e he accepted all: retire REQ-cmd-comment-001 and 002 here, reword
the site rows here, file the configured-commands warning as a follow-up (filed as #893), keep the dead
helpers and qualify their contracts, and leave `init --help` unchanged. No runtime behavior changes.

Ruled out: making `change check` run the commands (that reverses SHIP-1.c and REQ-change-091); deleting
the dead runner, classifier and routing helpers (decision d); adding a warning (decision c, #893).
