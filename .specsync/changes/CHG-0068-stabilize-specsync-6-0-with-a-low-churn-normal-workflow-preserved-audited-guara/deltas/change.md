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
- An added, removed, or replaced stable-scope item requires renewed approval, and status explains
  each criterion, affected spec/path, dependency, supersession obligation, or changed intent in
  plain language.
- Checking off an already-approved task records implementation progress without changing either
  scope or execution digests; changing task text preserves scope but stales execution evidence.
- Every status result prints exactly one explicit next action.
- Expected missing-history ancestry probes never leak raw Git fatal diagnostics into status output.
- Explicit `--strict`, project policy, or release/security classification adds full-history,
  full-suite, security, or release validators to the same verification evidence without changing
  the state machine, workflow, approval count, commands, finalization, archive, or layout.
- Existing two-approval records remain readable and verifiable without reinterpretation or resigning.
- Workflow-v2 adoption records one immutable project cutoff at the stable comparison-base ancestor
  when available; its introduction remains valid after squash/rebase, and a workflow-v1 record
  remains eligible only when that exact ID/version with omitted or explicit version-1 origin
  existed at the trusted cutoff, so omitting both version fields before first reachability cannot
  route a new change through legacy commands.
- Every bounded workflow-v2 baseline-touching commit and readable parent retains the exact
  introduction bytes, so rewrite→restore history cannot conceal a changed cutoff.
- Workflow-origin history boundedly follows every reachable canonical dated archive state path for
  the exact change ID, so archive→reopen→rearchive moves preserve the immutable creation anchor.
- The historical CHG-0068 source-definition preimage remains explicitly unavailable and no
  equivalence proof is claimed; one compile-time allowlist freezes its exact source approval,
  adoption commit/blob, stable-scope digest, authorization, and non-material classifications;
  missing immutable anchor history fails closed.

### REQUIREMENT REQ-change-044

The lifecycle SHALL finalize and archive a change on its implementation PR through one
metadata/archive-only commit without repeating implementation validation.

Acceptance Criteria

- `change finalize` requires the approved implementation parent to have every required green check.
- The finalization child may change only exact approved lifecycle/archive paths and must preserve
  code, canonical spec, requirements, tests, configuration, and delivery-tree relationships.
- Finalization applies semantic deltas, writes accepted state, validates bidirectional ownership,
  and moves the same package to `.specsync/archive/changes/YYYY-MM-DD-<id>/` transactionally.
- Interrupted terminal writes recover from the transaction journal, and a surviving exact archive
  subtree authenticates squash/rebase integration when the implementation commit is unreachable.
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
- The result binds the implementation parent commit, those input digests, an explicit pass/block
  verdict, a stable reviewer claim distinct from the scope approver, append-only attempt history,
  and required GitHub Actions check provenance.
- Native review recording and finalization run the same every-parent verification-freshness
  validator as project checking, and every persisted review attempt revalidates that its reviewer
  is distinct from the scope approver bound to that attempt's contract digest.
- Every intervening commit is inspected against every parent; implementation changes, including
  change-then-revert history, stale the review, while the metadata/archive-only finalization commit
  does not rerun or stale it; native and hosted validation share one committed limits document.
- Finalization fails when a required scoped review is missing or blocking.
- Status states when review is needed and directs the user to open or update the PR so the configured
  scoped-review check runs.
- An interrupted archive move resumes safely from the dated destination without requesting another
  approval.

## MODIFIED

### REQUIREMENT REQ-change-013

The lifecycle SHALL reject untrusted or corrupt persisted workspace identity, scope, approval,
history, verification, and scoped-review evidence before using it, with one
environment-independent verification-freshness decision.

Acceptance Criteria

- Loaded change IDs match their requested workspace and remain a single validated component.
- Persisted affected spec names are validated before delta paths are constructed.
- Unreadable or malformed historical tombstone deltas and approval ledgers fail closed.
- Verifying workspaces require passed evidence, matching effective contract/project-input digests,
  and a current independent scoped review when finalization is next.
- A descendant verification commit remains current only when every intervening commit and every
  parent edge changes exactly `state.json`, `verification.json`, `verification-attempts.json`,
  `review.json`, or `review-attempts.json` under a canonical active-change ID and the persisted
  state, verification, latest-attempt, and scoped-review evidence remains consistent.
- Source-change-then-revert history, a review file mixed with a delivery change, ambiguous merges,
  nonancestor history, malformed paths, and any broader volatile or lifecycle path fail closed.

### REQUIREMENT REQ-change-016

The lifecycle SHALL preserve accepted closing evidence and supported verifying and scoped-review
evidence across repository-integrated commits without accepting unintegrated, altered, or
historically tainted evidence.

Acceptance Criteria

- Normal verification-commit ancestry remains mandatory proof and uses identical local and CI
  semantics.
- Every intervening commit is inspected against every parent with NUL-delimited portable paths; a
  net tree diff cannot hide a governed change and later revert.
- Only `state.json`, `verification.json`, `verification-attempts.json`, `review.json`, and
  `review-attempts.json` beneath canonical active-change IDs may follow verification without
  invalidating it; archive, approvals, tasks, definitions, sequence, hashes, locks, configuration,
  policy, specs, source, tests, build, and cache paths are rejected.
- Matching effective contract and project-input digests plus consistent state, verification,
  latest-attempt, and scoped-review evidence remain mandatory.
- A squash fallback for accepted closing evidence still requires matching scoped inputs and an
  unchanged accepted workspace integrated on the remote default branch.
- Unintegrated heads, changed scoped inputs, stale contracts, mismatched closing approvals,
  nonancestor evidence, and ambiguous merges fail closed.
- Digest fields remain versioned, domain-separated, and length-framed; binary bytes, topology, and
  executable modes remain exact.

### REQUIREMENT REQ-change-029

Acceptance evidence SHALL preserve historical validity across valid later sequence claims without
weakening current sequence-ledger integrity.

Acceptance Criteria

- Creating a later valid lifecycle record does not stale an earlier accepted record solely because
  the sequence ledger advanced.
- Historical reconstruction uses the earlier owner and includes only collision acknowledgements
  whose sequence is not later than that owner.
- When acknowledged legacy collision members signed one canonical committed ledger for their shared
  sequence, reconstruction reuses those exact historical bytes instead of substituting each
  member's ID.
- The current sequence owner remains bound to the exact current ledger content.
- Malformed claims, claims without a workspace, non-maximum claims, duplicate sequences, and
  invalid collision acknowledgements fail closed.
- Every covered path other than a valid later-owned sequence ledger remains acceptance-digest
  input.
