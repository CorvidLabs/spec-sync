---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: plan
---

# Plan

## Approach

Add one audited verb, `change close-late`, and one terminal state, `closed_late`, as REQ-change-143
specifies. It closes changes that merged long ago and were never archived, records that they were closed
late and never re-checked, and takes none of the evidence reads that fail on such changes. Failing tests
come first.

## Out of scope

- The reopen bounded-read overflow (#976, the next PR, REQ-change-144).
- A listing of every archive that fails authentication (#833 fix 3, #831).
- `--all-merged`, deferred by D3.
- Any GitHub API check that a PR merged.
- Any change to how accepted or archived evidence is authenticated.
- No tag, release or crates.io publish.

## Steps

1. Write the failing tests: the eight groups in testing.md. Record that they fail on main f7180c34.
2. After scope approval, add `ChangeState::ClosedLate` and decide every exhaustive match.
3. Add the verb, the merge proof, the record, the all-or-nothing write and `--dry-run`.
4. Add the reporting in status, ship-status, JSON and audit.
5. Update the contracts: REQ-change-143 (delta), the cmd_change and cli_args Public API rows, the change.3md
   lesson plane, the site docs (workflow, cli), AGENTS.md quick reference and docs/HLD.md, and CHANGELOG.
6. Run `change check --commit`, `change audit --strict`, the verify lane on the pinned rust 1.89, and
   pre-push.

## Risks

- **Large surface.** The new state touches about 116 `ChangeState::Archived` references. The exhaustive
  matches force a decision at each one, and an over-broad arm could let `closed_late` pass as archived.
  Mitigation: tests 6 and 7 assert that every verified path refuses it.
- **Older binaries.** Versions before this one fail closed on `closed_late` (D1, accepted). Repositories
  must pin a release that has it before running it.
- **Merge proof.** This relies on the local remote-tracking ref. A stale fetch can refuse a change that has
  merged, which fails safe. It cannot admit an unmerged one unless the ref names a branch that holds the
  package.

## Constraints consulted

- SHIP-2.c: a recorded name is a claim, not proof.
- SHIP-6 and RECOVER-2: a closure that counts as verified still needs fresh verification and review. This
  verb never claims that.
- The archive-first CI lesson in change.3md: closing late moves the workspace out of `.specsync/changes/`,
  so the Archive job sees no open change.
- No gate, bound, timeout or existing assertion is loosened.
