## MODIFIED

### REQUIREMENT REQ-change-050
SpecSync SHALL leave a newly initialised project able to complete its own lifecycle, and
SHALL treat an active-change directory that contains no `state.json` and no regular file at
any depth as not an active change in this working tree rather than as corruption. A directory
that holds a regular file but no `state.json` is damaged and still open.

Acceptance Criteria

- Fresh `init` writes SDD off with an empty `verification_commands` list; `specsync check` is
  the next step and does not need a project test command.
- A change directory with no regular file at any depth and no `state.json` is a husk and is
  skipped by active-change discovery, so `change new` succeeds on a branch that does not
  contain an earlier change.
- A change directory that holds a regular file but no `state.json` is reported unreadable,
  with a reason that says the state file is missing, and does not abort enumeration of its
  siblings. Sequence enumeration refuses that directory rather than skipping it.
- Every other read error, including an unreadable or malformed `state.json`, still fails closed.
- Verification exposes a lock-free body so a caller already holding the project lock can
  re-run it without deadlocking on the non-reentrant lock.

### REQUIREMENT REQ-change-068
Enumerating active changes SHALL return what could be read and what could not, as separate facts, so that no caller can mistake an unreadable workspace for an absent one.

Acceptance Criteria
- The roster reports readable records and unreadable workspaces separately, and each unreadable entry carries the workspace identity and a reason naming the offending path.
- A workspace that cannot be read does not abort enumeration: its healthy siblings are still returned.
- A failure that leaves no partial truth to report — the changes directory itself being unreadable — remains a hard error rather than an empty roster.
- A directory with no state file and no regular file at any depth is still skipped rather than reported unreadable, because a husk left by a branch switch is not an active change here. A directory that holds a regular file but no state file is reported unreadable, with a reason that says the state file is missing, and does not abort the roster.
- The plain record list used by digest, ledger and successor computations continues to fail closed on any unreadable workspace, since a silently short roster is worse there than a hard error.
- A project with no active changes still yields an empty roster with nothing unreadable.

### REQUIREMENT REQ-change-105
The system SHALL refuse approval of a selected plan until that plan says what will be done, what will not be done, the steps, the risks, and which existing constraints are honored. The system SHALL NOT add a plan to a change the interview did not select one for.

Acceptance Criteria
- A selected plan is incomplete while Approach, Out of scope, Steps, Risks, or Constraints consulted is missing or has no substantive text.
- A required section ends at the next level-two heading. Prose under a later heading does not fill the empty section above it.
- Headings, blank lines, HTML comments including those that span lines, and placeholder TODO lines are not substantive text.
- Headings and text inside a fenced code block are not part of the plan. A plan whose required headings appear only inside a fence is incomplete.
- A plan with substantive text under each of those headings is complete, once the generic artifact TODO check also passes.
- A bug fix and a documentation change that did not select a plan are not given one by this rule.
- Status names the plan file while it is incomplete, using the same predicate as approval.
