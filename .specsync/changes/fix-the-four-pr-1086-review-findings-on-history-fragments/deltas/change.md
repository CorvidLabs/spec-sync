## MODIFIED

### REQUIREMENT REQ-change-159

When `change check` materializes an approved delta into a module that does not yet record the change, SpecSync SHALL record the change in a history fragment, `history/<change-id>.md` beside the module's spec (REQ-change-160), instead of adding a Change Log row and bumping `version:` (#1084). It SHALL NOT write the spec's `version:` or its `## Change Log` section: a table written before this requirement stays as it is, no row rewritten, and new history goes only to fragments. A delta item that targets the `## Change Log` section SHALL be refused by `change approve`, `change check`, verification and acceptance, naming the item, and a refused check or verification SHALL record its refused attempt; an archived change is not re-judged. A module records a change when the change's fragment exists or, for a change an earlier SpecSync materialized, when a cell of the module's Change Log table begins with `<change-id>:`, after any leading `*`, `_` or backtick; the same matcher decides whether a change is recorded twice, and a module that records the change gets no second record. The fragment SHALL be one of the change's own canonical files, as the spec and its companions are: `change check --commit` stages it, path coverage and acceptance ownership count it, and a send-back that drops the module names it for removal.

Acceptance Criteria
- After `change check` of an approved change to `auth`, `specs/auth/history/<id>.md` holds the change id, today's UTC date and the change title, and `auth.spec.md` keeps its `version:` and its Change Log byte for byte.
- A second `change check` writes nothing; a module whose Change Log already names the change gets no fragment.
- `change check --commit` commits the fragment.
- Two changes to the same module, each checked and committed on its own branch from one base, merge with plain `git merge` and no merge driver configured, with no conflict in that module's spec.
- A change re-scoped by a send-back so that it no longer declares a module it wrote a fragment for is refused at its next check, naming the fragment to remove.
- A delta with a `SPEC SECTION Change Log` item is refused by `change approve`, naming the section and `history/`, and nothing is written.
- A row reformatted as `**<id>:**` still records the change, and with a fragment it is a double record; a row for `one-session` does not record `session`, and the id mentioned in another row's prose records nothing.
- A `change check` refused for a Change Log item records a refused attempt in `verification-attempts.json`.

### REQUIREMENT REQ-change-160

A history fragment SHALL be a UTF-8 file named `<change-id>.md` in the `history/` directory beside a module's spec. It SHALL open with YAML frontmatter holding `change:`, a string equal to the file name without `.md`, and `date:`, a `YYYY-MM-DD` date, followed by a one-line, non-empty summary; other frontmatter keys are allowed and ignored. SpecSync SHALL write `change:` as a double-quoted YAML string and SHALL read a double-quoted, single-quoted or plain value, so every change id it accepts round-trips and a fragment written unquoted still reads. A module's history SHALL be the rows of its Change Log table as written, followed by its fragments in date order and then change-id order.

Acceptance Criteria
- The fragment `change check` writes parses back to the same change id, date and title, including ids such as `true`, `'foo'` and `say "hi"`.
- A fragment written with a plain `change:` still parses.
- Fragments with the same date are ordered by change id.
