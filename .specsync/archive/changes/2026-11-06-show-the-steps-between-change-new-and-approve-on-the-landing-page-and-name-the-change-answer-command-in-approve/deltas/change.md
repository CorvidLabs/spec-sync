## ADDED

### REQUIREMENT REQ-change-133

When `change approve` refuses a draft because it has no acceptance criterion or its deterministic interview is unfinished, the refusal SHALL name the `specsync change answer` command that supplies the missing answer. The approval guard SHALL refuse exactly the definitions it refused before.

Acceptance Criteria
- With no acceptance criterion, the refusal names `specsync change answer <id> acceptance_criteria "<observable outcome>"`.
- With interview questions still open, the refusal names every remaining question and the `specsync change answer <id> <question> <answer>` command for the first, giving the question's choices (such as `<yes|no>`) as the answer placeholder.
- Running the named command with its placeholder filled moves approval on to the next refusal or to the selected-artifact refusal.
- A definition that approval refused before is still refused.
