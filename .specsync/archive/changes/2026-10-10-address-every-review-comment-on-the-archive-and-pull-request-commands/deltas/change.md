## MODIFIED

### REQUIREMENT REQ-change-105
The system SHALL refuse approval of a selected plan until that plan says what will be done, what will not be done, the steps, the risks, and which existing constraints are honored. The system SHALL NOT add a plan to a change the interview did not select one for.

Acceptance Criteria
- A selected plan is incomplete while Approach, Out of scope, Steps, Risks, or Constraints consulted is missing or has no substantive text.
- A required section ends at the next level-two heading. Prose under a later heading does not fill the empty section above it.
- Headings, blank lines, HTML comments including those that span lines, and placeholder TODO lines are not substantive text.
- A plan with substantive text under each of those headings is complete, once the generic artifact TODO check also passes.
- A bug fix and a documentation change that did not select a plan are not given one by this rule.
- Status names the plan file while it is incomplete, using the same predicate as approval.

### REQUIREMENT REQ-change-106
The system SHALL show, when a change is opened, shown, or asked its status, the lessons already written for the modules it touches and the project's principles when a principles file is configured. The text result and the structured result SHALL carry the same briefing. A module context that holds only its generated scaffold SHALL NOT be presented as knowledge. The briefing SHALL be bounded and SHALL name the file for anything left unread. An unreadable context or principles file SHALL NOT fail the command.

Acceptance Criteria
- The briefing lists each affected module context that has substantive lines, with the path, the substantive line count, a bounded excerpt, and whether the excerpt was cut.
- The context path is the directory of the module's canonical spec, including a non-default specs directory or a registry mapping.
- A multiline HTML comment is not a lesson.
- A scaffold-only context is absent.
- A configured principles file is named, with a bounded excerpt, or marked missing when it cannot be read.
- No principles file produces no principles entry.
- The briefing is computed and is not stored on the change record.
