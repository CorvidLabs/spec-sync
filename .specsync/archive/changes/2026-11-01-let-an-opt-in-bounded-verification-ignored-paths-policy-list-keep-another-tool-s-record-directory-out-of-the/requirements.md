---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: requirements
---

# Requirements

## Numbering

The IDs are provisional. At draft time (origin/main a80748ff) the living tree ends at
REQ-change-112 and REQ-cmd-change-022, and the parallel drafts claim:

- `0xleif/feat/status-delta-requirements` (#826): REQ-change-113 and REQ-cmd-change-023;
- `0xleif/fix/generated-markdown-no-em-dash` (#827): REQ-change-116 (it has moved off 113 since
  an earlier revision, when it collided with #826).

This draft claims REQ-change-114, REQ-change-115 and REQ-cmd-change-024, so the three drafts do
not collide with each other today. That does not settle numbering. Approval refuses `## ADDED`
for an ID already in the living tree, so the final numbers are decided at approval, in landing
order: whichever package is approved after another has landed takes the next free IDs and
renumbers its deltas, requirements and test-evidence table together.

### REQ-change-114

The system SHALL leave every file under a directory prefix listed in `.specsync/sdd.json`
`verification_ignored_paths` out of the verification project-input digest, whether the file is
tracked or untracked, and SHALL leave the scoped-review descendant walk unchanged. With the list
absent or empty, the digest and every persisted byte SHALL be unchanged.

Acceptance Criteria
- With `[".chough/"]` committed and the change verified, adding, editing, deleting or committing a
  file under `.chough/` before the review keeps verification current, and status names
  `change review`.
- `change review` records a passing review and `change finalize` accepts it. An uncommitted edit
  under a listed prefix after the review leaves the review current.
- Any commit after the review, including one that touches only listed paths, still stales the
  review through the descendant walk.
- Edits or commits outside every listed prefix, and a path equal to an entry without its trailing
  slash, still stale verification and review.
- Adding, editing or removing an entry stales verification and review.
- No persisted record gains a field. With the list absent or empty, the digest,
  `verification.json`, `review.json` and status JSON are byte-identical to before.
- The following are unchanged:
  - `ignored_paths`, `meaningful_paths` and coverage,
  - `config.toml` `exclude_dirs` and `exclude_patterns`,
  - acceptance-input digests and manifests,
  - lifecycle commit staging.

### REQ-change-115

The system SHALL accept a `verification_ignored_paths` entry only when it is a bounded, portable,
project-relative directory prefix that overlaps no specs, sources, SDD policy inputs or
`meaningful_paths` scope. It SHALL refuse an invalid list in every lifecycle verb that loads SDD
policy, and SHALL refuse, for a change, an entry that overlaps that change's declared
`affected_paths`.

Acceptance Criteria
- An entry is refused, with a message naming it and the rule, when any of these holds:
  - it does not end in `/`;
  - it contains a glob or pattern character;
  - it is absolute, has an empty, `.` or `..` component, a backslash or a control character;
  - it is or lies under a symlink on disk;
  - it exceeds 128 bytes;
  - it duplicates or nests in another entry;
  - the list has more than 8 entries.
- An entry is refused when it overlaps, ignoring ASCII case, any of these:
  - `.specsync/`,
  - the configured specs tree,
  - a `source_dirs` entry,
  - the configured `schema_dir`,
  - any `meaningful_paths` scope,
  - an SDD policy file, the `principles_file` or a `custom_artifacts` template,
  - a file listed in a spec's `files:`,
  - a path-form spec `depends_on` entry (one naming a path, not a bare module name or a
    cross-project reference),
  - a file named in a spec's `Consumed By` table.

  An `ignored_paths` scope that covers the entry does not relax this. `tests/fixtures/` and
  `.github/workflows/` are refused under the default policy whatever `ignored_paths` says.
- In a project rooted in a subdirectory of its Git repository, an entry excludes only paths under
  that project's own copy of the prefix.
- `change new`, `change answer`, `change approve`, `change check`, `change review`,
  `change finalize`, `change ship` and `change audit` refuse an invalid list as an invalid SDD
  policy. `change status` makes fixing the entry its one next action. `change ship-status`
  reports it as a blocker naming the entry. No evidence is recorded against it.
- A change whose `affected_paths` overlap an entry is refused at create, at the paths answer, and
  at approve, check, review and finalize. Other changes are unaffected.
- `specsync check` does not read the list.

### REQ-cmd-change-024

`change status` and `change check` SHALL disclose a non-empty `verification_ignored_paths` list
in text and status JSON, together with the limit that SpecSync cannot tell whether project tests
or CI read files under those prefixes, and `change ship-status` SHALL name an invalid entry as a
blocker.

Acceptance Criteria
- `change status --json` carries `summary.verification_ignored_paths` when the list is non-empty,
  and omits it otherwise.
- `change ship-status` lists an invalid `verification_ignored_paths` entry as a blocker that names
  the entry and the rule, not as stale verification.
- Text status and check print `Verification ignores: <entries>` and the standing note only when
  the list is non-empty.
- With the list absent or empty, text and JSON output are unchanged.

### Contract

The `change` spec's Contract gains item 24, restating REQ-change-114 and REQ-change-115, and
item 6 is amended in the same restated section to name the exception: the governed inputs that
verification and scoped review bind are every project input except files under a listed prefix.
Item 6's descendant-walk rule is not changed. `cmd_change`'s Contract gains item 12, restating
REQ-cmd-change-024. The full restated sections are in `deltas/change.md` and
`deltas/cmd_change.md`.
