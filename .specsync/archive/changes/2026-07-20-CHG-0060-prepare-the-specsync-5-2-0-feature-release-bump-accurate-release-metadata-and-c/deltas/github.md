## ADDED

### REQUIREMENT REQ-github-004

The maintained GitHub Action SHALL promote the 5.2.0 release through an immutable exact-version
ref whose default binary version synchronizes only after exact-version artifacts pass
supported-platform verification, with the floating major ref following the same contract.

Acceptance Criteria

- The composite Action's default and maintained consumer pins read exactly 5.2.0 once the
  accepted release commit lands on main.
- The immutable `v5.2.0` Action ref resolves to the integrated release commit after publication.
- The floating `v5` ref moves to 5.2.0 only after pinned consumers pass on Linux, macOS, and
  Windows.
- A failed exact-version asset or Action smoke test leaves the floating ref and prior default
  unchanged.

## MODIFIED

### SPEC SECTION Invariants

1. `fetch_issue` always tries `gh` CLI first, falls back to REST API only if `gh` is unavailable
2. `fetch_issue_api` requires `GITHUB_TOKEN` environment variable; returns error if unset
3. `fetch_issue_api` uses a 10-second HTTP timeout
4. Issue state is normalized to lowercase (`"open"` / `"closed"`)
5. `create_drift_issue` requires `gh` CLI — no REST API fallback for issue creation
6. `detect_repo` handles both SSH (`git@github.com:`) and HTTPS (`https://github.com/`) remote URLs
7. `resolve_repo` prefers explicit config over auto-detection
8. `verify_spec_issues` classifies each issue as valid (open), closed, not_found, or error
9. Action defaults and maintained consumer pins advance to an exact release version only through
   an accepted release change, and floating-ref promotion waits for supported-platform
   verification of the exact-version artifacts.
