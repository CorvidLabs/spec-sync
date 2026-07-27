## MODIFIED

### REQUIREMENT REQ-cmd-import-001

The import command SHALL create non-overwriting draft specs from supported single and batch sources
with deterministic companion generation.

Acceptance Criteria

- Single and batch GitHub imports require explicit `GITHUB_TOKEN` and execute typed in-process REST
  reads without consulting authenticated `gh` state.
- Every GitHub REST operation is bounded to 10 seconds.
- `--all-issues` follows strict encoded pagination for at most 100 pages of 100 provider entries,
  rejects an oversized page before item parsing, and fails on malformed links, duplicate issue
  IDs, or a continuing next page at the cap.
- A pagination failure is an error, never a successful partial import.
- Every single and batch output module name passes shared portable validation before filesystem
  paths are joined or created.
- Batch item errors do not stop later items, but the final truthful summary is followed by exit 1
  whenever any error occurred.

### SPEC SECTION Invariants

1. GitHub import reads never launch a provider subprocess.
2. Single imports use the shared typed issue-detail contract.
3. Batch imports use strict, bounded, complete pagination.
4. Missing tokens, malformed responses, inaccessible repositories, transport failures, timeouts,
   pagination ambiguity, and cap truncation fail closed.
5. Unsafe/reserved/overlong output names create no directory or file.
6. Partial batch success never produces a false-green exit status.
