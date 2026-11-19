## ADDED

### REQUIREMENT REQ-validator-048

`specsync check` SHALL validate the history fragments of every spec it validates (REQ-change-160): every entry in the spec's `history/` directory SHALL be a regular `<change-id>.md` file in the fragment format, and a change SHALL NOT be recorded both by a fragment and by a row of the same spec's Change Log. Each violation SHALL be an error that names the file and what is wrong. A spec with no `history/` directory has nothing to validate.

Acceptance Criteria
- A fragment whose `change:` differs from its file name, whose `date:` is not a date, whose summary is empty or longer than one line, or which has no frontmatter, and an entry not named `*.md`, each fail `specsync check`, naming the file.
- A fragment whose change also has a row in the same spec's Change Log fails `specsync check`.
- Valid fragments pass `specsync check --strict`.
