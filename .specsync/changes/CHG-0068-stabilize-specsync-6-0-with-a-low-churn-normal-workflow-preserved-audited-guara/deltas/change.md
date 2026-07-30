## ADDED

### REQUIREMENT REQ-change-043

The verified lifecycle SHALL provide one discoverable workflow and one file layout with one human
scope approval for every new change.

Acceptance Criteria

- The path is `change new` → one `change approve` → implement → `change check` → ordinary PR
  review → `change finalize` → GitHub merge.
- There is no lifecycle-mode selection, second SpecSync approval, closing approval gate, alternate archive
  layout, or SpecSync merge command.
- Scope approval binds stable intent, acceptance criteria, public-contract/risk declarations, and
  affected spec/path/dependency/supersession scope—not implementation, test/evidence,
  semantic-delta materialization, canonical materialization, or lifecycle metadata.
- Non-material execution/evidence changes preserve scope approval, invalidate their separate
  execution digest, and require fresh automated validators plus the one scoped review.
- A demonstrable scope expansion requires renewed approval, and status explains each added
  criterion, affected spec/path, dependency, supersession obligation, or changed intent in plain
  language.
- Checking off an already-approved task records implementation progress without changing either
  scope or execution digests; changing task text preserves scope but stales execution evidence.
- Every status result prints exactly one explicit next action.
- Explicit `--strict`, project policy, or release/security classification adds full-history,
  full-suite, security, or release validators to the same verification evidence without changing
  the state machine, workflow, approval count, commands, finalization, archive, or layout.
- Existing two-approval records remain readable and verifiable without reinterpretation or resigning.

### REQUIREMENT REQ-change-044

The lifecycle SHALL finalize and archive a change on its implementation PR through one
metadata/archive-only commit without repeating implementation validation.

Acceptance Criteria

- `change finalize` requires the approved implementation parent to have every required green check.
- The finalization child may change only exact approved lifecycle/archive paths and must preserve
  code, canonical spec, requirements, tests, configuration, and delivery-tree relationships.
- Finalization applies semantic deltas, writes accepted state, validates bidirectional ownership,
  and moves the same package to `.specsync/archive/changes/YYYY-MM-DD-<id>/` transactionally.
- The lightweight archive lane validates parent checks, diff classification, unchanged tree,
  archive integrity, ownership, and finalization digest and reports success to required CI.
- Product tests and independent scoped review are not rerun for a valid archive-only child.
- `change finalize` makes the PR ready; GitHub alone performs the merge.

### REQUIREMENT REQ-change-045

Lifecycle validation SHALL reuse a deterministic invocation-scoped snapshot and bounded evidence
queries without weakening fail-closed historical conclusions.

Acceptance Criteria

- Active/archive records, canonical owners, Git comparison state, candidate entries, and completed
  terminal evidence are loaded or computed at most once per invocation key.
- Git and evidence queries have deterministic bounds independent of overlapping path scopes.
- Dependency and successor graphs use stable ordering.
- Canonical owner batches are validated in one pass.
- Warm and cold validation return identical errors, warnings, path coverage, and evidence validity.

### REQUIREMENT REQ-change-046

Agent-authored changes SHALL receive one independent scoped review of implementation evidence before
finalization.

Acceptance Criteria

- Review input contains only the change package, implementation diff, canonical semantic delta, and
  targeted evidence.
- The result binds the implementation parent commit and those input digests.
- Implementation changes stale the review; the metadata/archive-only finalization commit does not
  rerun or stale it.
- Finalization fails when a required scoped review is missing or blocking.
- Status states when review is needed and directs the user to open or update the PR so the configured
  scoped-review check runs.
