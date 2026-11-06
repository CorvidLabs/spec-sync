---
change: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
artifact: design
---

# Design

One rule, REQ-change-023: `change check` does not execute `.specsync/sdd.json`
`verification_commands`. Every other clause either defers to it by ID or is retired.

| Clause | Was | Becomes | Why |
|---|---|---|---|
| REQ-change-015 | quiet reporting mode that executes every configured command | REMOVED | REQ-change-058 says that mode SHALL NOT exist |
| REQ-change-019 AC3 | "a failed configured command" is one of three distinct diagnostics | the two evidence diagnostics stay distinct and neither names a command | no command runs, so none can fail |
| REQ-change-027 | configured verification rejects re-entry; native commands "execute once" | a lifecycle command refuses an inherited context; SpecSync sets it on no child | only the inherited-marker guard is live |
| REQ-change-030 | preserves "native verification commands"; Cargo identity criteria read as live | body drops it; Cargo criteria describe the retained classifier no command consults | the classifier is dead code, kept by decision d |
| REQ-change-061 AC2 | "populating them as initialization instructs" | "editing the list"; init writes it empty | init gives no such instruction |
| REQ-change-069 | commands "a change receives" | commands the retained routing helper selects; none executed | routing is test-only |
| REQ-cmd-comment-001, 002 | comments fold SDD lifecycle failures | REMOVED | REQ-cmd-comment-004 reverses them (#543) |
| REQ-cmd-comment-003 AC1 | command output captured away from stdout | comment executes no configured command | nothing to capture |
| cmd_comment scenario | the command still executes | the command is not executed | same |
| cmd_comment top-level AC, context.md and 3md status | still execute; change check streams it | executes none; neither spawns it | matches the integration test |
| site comparison rows | SpecSync runs configured tests; no-shell verification | your CI runs the tests; verification runs no configured command | public guidance makes the same promise |

Requirement edits go through the semantic deltas, so `change check` materializes them and the
stale-MODIFIED gate protects them. Companion prose (the top-level list in cmd_comment
requirements.md, context.md, testing.md and lessons) and site pages are direct edits inside the
declared paths. History stays as it is: change-log rows and earlier lesson planes describe what
was true then.
