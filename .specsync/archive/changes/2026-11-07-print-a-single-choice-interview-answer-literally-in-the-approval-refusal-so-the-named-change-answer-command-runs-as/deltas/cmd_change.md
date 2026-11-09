## ADDED

### REQUIREMENT REQ-cmd-change-answer-hint-001

For a draft with an unanswered interview question, text and JSON `change status` SHALL name as the next action the same `specsync change answer` command that `change approve`'s refusal names for that question (REQ-change-133), including `constraints`, which is asked once the rest of the interview is answered (DOCS-3.c).

Acceptance Criteria
- After each refusal the landing-page draft gets, text `Next:` and JSON `summary.next_action` name the command that refusal named.
- A yes-or-no question is named with `<yes|no>`, not `<answer>`, and `constraints` as `constraints none`.
- Text status for a draft whose only open question is `constraints` names that answer, not the artifacts step.
