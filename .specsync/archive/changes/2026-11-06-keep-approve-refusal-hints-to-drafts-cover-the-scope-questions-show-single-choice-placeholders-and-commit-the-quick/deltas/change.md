## MODIFIED

### REQUIREMENT REQ-change-133

When `change approve` refuses a draft because its definition is unfinished (no acceptance criterion, no affected path, no affected spec without a justified no-spec change, or interview questions still open), the refusal SHALL name the `specsync change answer` command that supplies the missing answer. A record past draft SHALL NOT be told to run `change answer`, which refuses outside draft. The approval guard SHALL refuse exactly the definitions it refused before.

Acceptance Criteria
- With no acceptance criterion, a draft's refusal names `specsync change answer <id> acceptance_criteria "<observable outcome>"`.
- With no affected path, a draft's refusal names `specsync change answer <id> affected_paths "<path>"`; with no affected spec and no justified no-spec change, `specsync change answer <id> affected_specs "<module>"`.
- With interview questions still open, the refusal names every remaining question and the `specsync change answer <id> <question> <answer>` command for the first, giving the question's choices as the placeholder (`<yes|no>`, `<none>`), or `"<answer>"` when it has none.
- Running the named command with its placeholder filled moves approval on to the next refusal or to the selected-artifact refusal.
- An approved, implementing or verifying record that fails the same checks is refused with the same condition text and no `change answer` command.
- A definition that approval refused before is still refused.
