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

## MODIFIED

### REQUIREMENT REQ-change-021

The lifecycle SHALL NOT append a row to a canonical Change Log table (REQ-change-159). When `specsync history` renders a history fragment as a row (REQ-cmd-change-037), it SHALL preserve the existing table's column schema.

Acceptance Criteria
- A `Version | Date | Changes` table renders a fragment as the frozen `version:` counted on by the fragment's position, the fragment's date, and `<id>: <summary>`, in that order.
- A `Date | Author | Change` table renders the fragment's date, `SpecSync`, and `<id>: <summary>`, in that order.
- A two-column `Date | Change` table renders the fragment's date and `<id>: <summary>`.
- A rendered row has the same number and order of cells as every recognized existing header; a column it does not recognize is left empty.

### REQUIREMENT REQ-change-092

Canonical materialization SHALL be decided from the canonical artefacts rather than from the
`canonical_applied` flag alone, so that every output materialization produces is present for the
delta bodies the change has currently approved.

Acceptance Criteria
- A semantic delta corrected after review and re-approved is materialized into the canonical spec
  by the next `change check`, and the superseded wording does not survive beside the correction.
- A module whose canonical spec carries the change's contract text but does not record the change,
  by its history fragment or by a Change Log row an earlier SpecSync wrote, receives the fragment
  on the next `change check`, and never a `version:` bump or a row (REQ-change-159). The record is
  not derivable from a delta digest, so re-applying the delta alone would not close this.
- A re-approval whose delta body is byte-identical writes nothing at all: the canonical spec,
  requirements and history fragment stay byte for byte as they were, and the module records the
  change exactly once. Re-materializing unconditionally is refused as a fix, because it would
  rewrite every canonical spec on every check.
- Re-materialization does not refuse the work its own earlier run performed: a `## REMOVED` item
  whose block is already absent reads as applied. That reading is available ONLY to a change that
  has already materialized once; on a first materialization every application refusal still fires
  unchanged, including removing a block that was never present.
- The refusal for a semantic delta that changed after its approval names `specsync change check`
  after `specsync change approve`. Approval binds the wording and only `check` puts it in the
  canonical spec, so a remedy naming approval alone walked the author into the silent skip.

### REQUIREMENT REQ-change-123

`change send-back` SHALL move an approved, implementing or verifying change back to draft so its scope questions can be answered again. It SHALL require a non-empty human actor and reason, SHALL append both, with the prior state, to `approvals.json` under `send_backs`, and SHALL cancel the definition approval recorded before it without removing any recorded approval, verification or other evidence. A definition approval recorded before the latest send-back SHALL NOT satisfy the definition gate or bind a scoped review; only a fresh `change approve` does. A scoped review recorded before the send-back SHALL stay in the append-only review history and SHALL still validate against the approval it bound; it SHALL NOT stop a fresh `change review` after re-approval, and `change status` SHALL report it stale rather than unreadable. A send-back SHALL record how many review attempts preceded it, and a scoped review recorded before the latest send-back SHALL NOT be current, even when the re-approved definition has the same digest; for a send-back recorded without that count, a review SHALL count only when its timestamp is later than the send-back's. A send-back the committed ledger already holds SHALL NOT be removable or editable to bring a cancelled approval back. A send-back committed in any revision of the ledger SHALL NOT be removable or editable later, and the project audit SHALL report one that was. The send-back SHALL list the spec items the change had already applied, including REMOVED items, with each block's text from before the change, and the modules that record the change: by its history fragment, or by a Change Log row an earlier SpecSync wrote. After a send-back, materialization SHALL converge on what the canonical tree already carries, so a kept REMOVED, ADDED or MODIFIED item is not applied twice and a kept ADDED item may change its body. `change check` and `change accept` SHALL refuse, naming the spec file and the item and saying what to restore, while an item the new scope no longer declares is still applied, or a module it no longer declares still records the change; for a history fragment the refusal names the file to delete.

Acceptance Criteria
- A verifying change sent back can have `affected_paths` answered again, is approved, checked, verified and accepted under its own ID, and an already-applied section is not applied twice.
- A missing actor or reason is refused, naming `--actor` or `--reason`, and nothing is written.
- The send-back record keeps the actor, reason and prior state; earlier approvals and `verification.json` are unchanged; a second send-back appends.
- Until re-approval the definition gate refuses, says the change was sent back, and names `specsync change approve <id>`.
- A draft, accepted or archived change cannot be sent back.
- A sent-back change that owns a spec can be finalized and its archive passes the project audit.
- A re-scope that drops an item the change already applied is refused by check, naming the spec file and the item, until that text is taken out of the spec.
- The definition approval event resolver refuses an approval that a send-back cancelled.
- Removing a committed send-back from `approvals.json` is refused and does not bring the cancelled approval back.
- After a send-back, a change with a REMOVED item materializes and checks again (0xLeif's repro), and a kept ADDED item with a new body replaces the change's earlier body.
- A dropped MODIFIED item is refused with advice to restore its pre-change text, quoting it; a dropped REMOVED item is listed and refused until the block is restored.
- A module dropped from `affected_specs` is refused while its history fragment, or a Change Log row an earlier SpecSync wrote, records the change, and the refusal names the fragment to delete.
- A send-back deleted in a later commit is refused by check and reported by the project audit.
- A change reviewed before its send-back is re-approved, re-checked and reviewed again, and finalize archives it with both review attempts in its history, the first unchanged (#914).
- After a send-back that changes the scope, `change status` reports the earlier review stale with the re-review next action, and `change ship` refuses it.
- A review bound only to an approval a send-back cancelled is never current.
- After a send-back with an unchanged scope and a `change check` without `--commit`, `change status` names `change review`, `change ship-status` reports the earlier review stale, and `change ship` refuses until a fresh review is recorded (#960).
- A send-back records the number of review attempts before it; a review recorded before the latest send-back is never current, and a send-back without that count compares timestamps.
