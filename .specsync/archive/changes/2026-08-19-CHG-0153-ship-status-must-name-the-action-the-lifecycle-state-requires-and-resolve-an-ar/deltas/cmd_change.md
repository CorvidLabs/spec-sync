## ADDED

### REQUIREMENT REQ-cmd-change-014

`change ship-status` SHALL name a next action the same binary will accept, and SHALL resolve a change's verification and review evidence from wherever that change currently lives.

Acceptance Criteria
- Outside the shipping window — draft, accepted, and archived — the printed next action equals the lifecycle next action, so a draft is told to answer its interview rather than to commit verification, and an archived change is told there is no further action.
- The next action is always a runnable command and never a restatement of a blocker; blockers continue to render on their own lines.
- An archived change reports the verification commit and the scoped review recorded in its archive package, rather than reporting none and missing because the artifacts were sought at the active workspace path it has left.
- Evidence resolution reuses the single active-or-archive workspace resolver rather than introducing a third path-construction idiom.
- An unreadable or unparseable archived verification artifact reports no verification evidence and leaves the command successful, so an already-damaged repository is not made harder to inspect.
