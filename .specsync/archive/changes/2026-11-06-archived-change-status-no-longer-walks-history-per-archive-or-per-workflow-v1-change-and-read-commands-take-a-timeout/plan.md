---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: plan
---

# Plan

## Approach

Two history walks that run once per archive or per workflow-v1 change become one walk per command. Both are cached the way the read cache already caches immutable answers (see design.md, A and B). Read commands get an optional deadline that stops at a phase boundary, flushes the completed verdicts, and names the phase and the resume command.

## Out of scope

- #831's dedicated `change verify-archive` command, and a signed finalize record. These are a later change built on this one.
- `change list` latency (12 s here), which is profiled next.
- A cold read that is independent of the archive count. That needs a batch reader shared across workers, or validating only the target archive's history. It is measured and reported here as a follow-up.
- Any change to lifecycle writes, gates, bounds or default timeouts.

## Steps

1. Commit the failing guard `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`, with its fixture trimmed to run quickly.
2. Add a failing cold guard: archived status Git processes must not grow with the number of archive-introducing commits.
3. B: the cutoff-tree verdict and its persistent cache entry, plus equivalence tests. The warm guard turns green.
4. A: the one-walk introduction index, plus an equivalence test against the per-commit index. The cold index guard turns green.
4a. C: seeded read-only scopes for prefetch workers, plus a cold archived-status guard (at most 150 Git processes for four more archives, was 250).
5. Deadline: the `read_deadline` module, phase checks, flushing the cache on timeout, `--timeout` and `SPECSYNC_TIMEOUT`, exit 124, text and JSON. Tests come first.
6. Extend `raven_sized_archived_status_benchmark` with workflow-v1 changes, and record before and after times on an unloaded Mac and on this repository.
7. `cli.md`, CHANGELOG, spec Public API rows; then `change check --commit` and the verify lane.

## Risks

- **Equivalence of B.** A path present at the cutoff that history never touched would be accepted. This can't happen, because the cutoff is reachable from HEAD and baseline history validation has already proven it. The equivalence tests cover a change absent at the cutoff, added after it, archived after it, and recorded under a different id.
- **`log -m --name-only` and merges.** It lists a merge's additions against every parent, as `diff-tree -m` did. A test with a merge that adds an archive pins this.
- **Deadline checks.** They must not leave a half-written cache. Only completed verdicts are flushed, through the existing atomic cache write.
- **Timings swing with load** on this shared Mac. Gating tests count Git processes, and timed numbers come from the on-demand benchmark.

## Constraints consulted

- SPEED-1.a (under 2 s on Raven), SPEED-1.c (cache invalidation rule), SPEED-1.d (Raven-sized benchmark).
- REQ-change-119, 121 and 124: existing archived-read and finalize guards stay.
- REQ-git-utils-010: batch reads through `HEAD` resolve through the memo.
- The read-cache rule: it is disabled for shallow, grafted, replace-ref and partial-clone repositories. B's cache entry follows the same rule.
