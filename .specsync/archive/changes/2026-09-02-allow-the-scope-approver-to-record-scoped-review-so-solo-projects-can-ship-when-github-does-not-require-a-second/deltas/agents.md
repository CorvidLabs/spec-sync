## ADDED

### REQUIREMENT REQ-agents-006

Generated SDD skill text SHALL describe scoped review as recordable by the same actor who approved the definition. It SHALL NOT instruct agents to invent a second human identity for solo work.

Acceptance Criteria

- The generated skill's lifecycle steps tell the agent to record `change review` with the human who signed off, including when that human also recorded definition approval.
- The skill does not require picking a second identity solely to satisfy SpecSync.
