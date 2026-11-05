## MODIFIED

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
- A demonstrable stable-scope change requires renewed approval, and status explains each added or
  removed criterion, affected spec/path, dependency, supersession obligation, or changed intent in
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
  when available; `change adopt` activates that baseline without rewriting an existing version-1
  policy, refuses before mutation when any existing workflow-v1 record is absent from the proposed
  cutoff, and atomically publishes policy, imports, report, and baseline so interruption or failure
  cannot partially activate workflow v2; every transaction target has a lossless UTF-8,
  platform-separator-safe journal identity and is confined beneath the project with symlink
  components rejected before and during publication. The lifecycle lock is likewise opened through
  a no-follow project capability before any metadata write. All subsequent changes use workflow
  v2, its introduction remains valid after squash/rebase, and a workflow-v1 record remains eligible
  only when that exact ID/version with omitted or explicit version-1 origin existed at the trusted
  cutoff.
- Every bounded workflow-v2 baseline-touching commit and readable parent retains the exact
  introduction bytes, so rewrite→restore history cannot conceal a changed cutoff and deleting a
  committed baseline—including one introduced only on a merged parent—cannot silently reactivate
  workflow v1.
- Workflow-origin history boundedly follows every reachable canonical dated archive state path for
  the exact change ID, so archive→reopen→rearchive moves preserve the immutable creation anchor.
- That history is read with repository-root pathspecs (`:(top,glob)` to discover dated archive
  paths, `:(top,literal)` to walk them). A project rooted in a subdirectory of its Git repository
  still sees every dated path and still refuses a workflow-version downgrade that a later commit
  restored.
- The one CHG-0068 adoption fails closed and requests full trusted history when its immutable
  allowlisted commit/blob anchor is unavailable.

### REQUIREMENT REQ-change-119

The system SHALL answer read-only lifecycle reads (`change status`, `change show`, `change list`, `change ship-status`, `change audit`) without re-deriving work whose inputs have not changed: inside one read scope an archived change's terminal evidence is computed for that change alone, scoped-review Git reads and archive lookups are memoized, and passing archived workflow-version history verdicts and archive introductions are reused across invocations from a cache under the Git directory that is keyed by content digest and invalidated by Git history.

#### Acceptance Criteria

- Output of every read is identical to a live read: same JSON, same refusals, same first error.
- A cached workflow-version history verdict is reused only when the SHA-256 of the archived `state.json` bytes, its archive path, the build identity and the repository prefix all match, the commit it was computed at is an ancestor of `HEAD`, and no commit in between touched that change's active or archived state paths. Anything else recomputes it live, and only passing verdicts are stored.
- That invalidation walk uses `:(top,literal)` pathspecs. A project rooted in a subdirectory of its Git repository does not reuse a cached pass across a commit that touched the change's state, including a workflow-version downgrade that a later commit restored.
- Archive introductions are cached per commit, which Git makes immutable, and only when every Git read behind them answered.
- The cache lives under the Git directory, is never a project input, is disabled in a shallow repository, and a failure to read or write it falls back to live reads. Mutating commands keep reading live.
- The Git processes one archived status read spawns do not grow with the number of other archived changes once the cache is warm, and a benchmark against a Raven-sized fixture (124 archived changes, 120 specs) times the read against the 2 second budget.
