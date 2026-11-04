## ADDED

### REQUIREMENT REQ-change-119

The system SHALL answer read-only lifecycle reads (`change status`, `change show`, `change list`, `change ship-status`, `change audit`) without re-deriving work whose inputs have not changed: inside one read scope an archived change's terminal evidence is computed for that change alone, scoped-review Git reads and archive lookups are memoized, and passing archived workflow-version history verdicts and archive introductions are reused across invocations from a cache under the Git directory that is keyed by content digest and invalidated by Git history.

#### Acceptance Criteria

- Output of every read is identical to a live read: same JSON, same refusals, same first error.
- A cached workflow-version history verdict is reused only when the SHA-256 of the archived `state.json` bytes, its archive path, the build identity and the repository prefix all match, the commit it was computed at is an ancestor of `HEAD`, and no commit in between touched that change's active or archived state paths. Anything else recomputes it live, and only passing verdicts are stored.
- Archive introductions are cached per commit, which Git makes immutable, and only when every Git read behind them answered.
- The cache lives under the Git directory, is never a project input, is disabled in a shallow repository, and a failure to read or write it falls back to live reads. Mutating commands keep reading live.
- The Git processes one archived status read spawns do not grow with the number of other archived changes once the cache is warm, and a benchmark against a Raven-sized fixture (124 archived changes, 120 specs) times the read against the 2 second budget.
