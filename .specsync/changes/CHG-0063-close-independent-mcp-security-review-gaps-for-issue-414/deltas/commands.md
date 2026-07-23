## ADDED

### REQUIREMENT REQ-commands-003

Drift-issue creation SHALL render untrusted text safely at both the command terminal and GitHub
issue boundaries.

Acceptance Criteria

- Repository-resolution failures, spec paths, returned issue URLs, and provider failures pass
  through the shared safe diagnostic renderer before terminal output.
- Terminal output does not preserve raw control characters, bidirectional formatting controls, or
  Unicode line/paragraph separators from untrusted values.
- The explicit GitHub creation helper sanitizes spec paths and validation errors separately for
  title text and Markdown body text.
- Sanitization does not change grouping: one drift issue is still attempted per spec, and an
  individual creation failure does not stop later specs.
- Public validation retains its rendered `Vec<String>` diagnostics contract. Private structured
  attribution and longest exact discovered-path matching preserve legal paths containing `": "`
  without exporting new command types.
