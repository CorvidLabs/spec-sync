---
module: change
---

## ADDED

### REQUIREMENT REQ-change-155

When an `ignored_paths` entry covers only paths that are always meaningful —
the configured specs tree, or an SDD policy file named directly — the policy
loader SHALL print a note on stderr naming the inert entry and the file to
edit. An entry that also covers other files (such as `.specsync/`, which
covers the change workspaces) SHALL NOT be named. Newly written default
policies SHALL NOT include an inert entry.

#### Acceptance Criteria

- A policy listing `specs/` prints the note at load; the note names the entry
  and `.specsync/sdd.json`.
- The default policy's `ignored_paths` no longer contains `specs/`.
- `.specsync/` produces no note.
