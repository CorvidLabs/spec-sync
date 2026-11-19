## ADDED

### REQUIREMENT REQ-change-159

When `change check` materializes an approved delta into a module that does not yet record the change, SpecSync SHALL record the change in a history fragment, `history/<change-id>.md` beside the module's spec (REQ-change-160), instead of adding a Change Log row and bumping `version:` (#1084). It SHALL NOT write the spec's `version:` or its `## Change Log` section: a table written before this requirement stays as it is, no row rewritten, and new history goes only to fragments. A module records a change when the change's fragment exists or, for a change an earlier SpecSync materialized, when the module's Change Log names the change; a module that records the change gets no second record. The fragment SHALL be one of the change's own canonical files, as the spec and its companions are: `change check --commit` stages it, path coverage and acceptance ownership count it, and a send-back that drops the module names it for removal.

Acceptance Criteria
- After `change check` of an approved change to `auth`, `specs/auth/history/<id>.md` holds the change id, today's UTC date and the change title, and `auth.spec.md` keeps its `version:` and its Change Log byte for byte.
- A second `change check` writes nothing; a module whose Change Log already names the change gets no fragment.
- `change check --commit` commits the fragment.
- Two changes to the same module, each checked and committed on its own branch from one base, merge with plain `git merge` and no merge driver configured, with no conflict in that module's spec.
- A change re-scoped by a send-back so that it no longer declares a module it wrote a fragment for is refused at its next check, naming the fragment to remove.

### REQUIREMENT REQ-change-160

A history fragment SHALL be a UTF-8 file named `<change-id>.md` in the `history/` directory beside a module's spec. It SHALL open with YAML frontmatter holding `change:`, equal to the file name without `.md`, and `date:`, a `YYYY-MM-DD` date, followed by a one-line, non-empty summary; other frontmatter keys are allowed and ignored. A module's history SHALL be the rows of its Change Log table as written, followed by its fragments in date order and then change-id order.

Acceptance Criteria
- The fragment `change check` writes parses back to the same change id, date and title.
- Fragments with the same date are ordered by change id.
