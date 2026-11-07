## MODIFIED

### REQUIREMENT REQ-change-133

When `change approve` refuses a draft because its definition is unfinished (no acceptance criterion, no affected spec without a justified no-spec change, no affected path, or interview questions still open), the refusal SHALL name the `specsync change answer` command that supplies the missing answer, and that command SHALL run as printed once any placeholder in it is filled. A record past draft SHALL NOT be told to run `change answer`, which refuses outside draft. A draft missing both its affected specs and its affected paths SHALL be refused for its affected specs first, the question `change status` asks first. The approval guard SHALL refuse exactly the definitions it refused before.

Acceptance Criteria
- With no acceptance criterion, a draft's refusal names `specsync change answer <id> acceptance_criteria "<observable outcome>"`.
- With no affected spec and no justified no-spec change, a draft's refusal names `specsync change answer <id> affected_specs "<module>"`; with no affected path, `specsync change answer <id> affected_paths "<path>"`; with neither, the affected-spec refusal comes first.
- With interview questions still open, the refusal names every remaining question and the `specsync change answer <id> <question> <answer>` command for the first. Two or more choices are the placeholder (`<yes|no>`). A single choice is the answer itself, so `constraints` is named as `specsync change answer <id> constraints none` and runs as printed, and the refusal adds that the lessons or principles that constrain the change can be named instead (DOCS-3.b). A question with no choices takes `"<answer>"`.
- Running the named command, with any placeholder filled, moves approval on to the next refusal or to the selected-artifact refusal.
- An approved, implementing or verifying record that fails the same checks is refused with the same condition text and no `change answer` command.
- A definition that approval refused before is still refused.
