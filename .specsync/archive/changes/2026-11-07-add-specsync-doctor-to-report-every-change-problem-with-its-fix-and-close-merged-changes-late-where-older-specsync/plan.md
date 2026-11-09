---
change: add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync
artifact: plan
---

# Plan

## Approach

Add `specsync doctor`. It lists every change problem with its fix, applies only the safe fixes, and applies a
late close only for named changes with an actor and a reason. The late close moves a merged package, byte for
byte, into `.specsync/archive/closed-late/`, which 6.0 to 6.2 never read, so older binaries neither trip on it
nor count it as checked. That behaviour is proven against real release binaries and kept proven in CI.
Failing tests come first.

## Out of scope

- The #976 bounded read, which is its own PR.
- `--close-late` for an unmerged change.
- Any `ChangeState` variant.
- Changing how accepted or archived evidence is authenticated.
- Reconciling the 6.2 plan sections of v1 records automatically (report only).
- No tag, release or crates.io publish.

## Steps

1. Write the failing tests in testing.md, and record that they fail on 1276e314.
2. After scope approval, add the closed-late location and its record, the lookup, the refusals and `change new`
   (REQ-change-143).
3. Add the doctor scan, its classes, `--fix`, `--dry-run` and `--close-late` (REQ-cmd-change-035), and the
   argument grammar (REQ-cli-args-025).
4. Add the compatibility script, the CI job and the fledge lane.
5. Update the contracts: the three deltas, cmd_change files, a change.3md lesson, the site docs (workflow, cli),
   the AGENTS.md quick reference, docs/HLD.md and CHANGELOG.
6. Run `change check --commit`, `change audit --strict`, the verify lane on the pinned rust 1.89, and pre-push.

## Risks

- **Old binaries see the folder after all.** The proof covers every 6.x release, and the CI test keeps it
  covered.
- **L1.** 6.x `change new` can mint a closed-late slug again. 6.3 refuses that slug, and doctor reports the
  duplicate.
- **L2.** A collision-member close edits `change-sequence.json`, which 6.x treats as meaningful, so the PR needs
  an open change that declares it. Doctor says so.
- **Network.** The CI job downloads release assets. A download failure fails the job loudly, never silently.
- **Size.** Doctor touches many read paths. Each finding kind has its own tests.

## Constraints consulted

- SHIP-2.c: a recorded name is a claim.
- SHIP-6 and RECOVER-2: a closure that counts as verified needs fresh verification and review. A late close
  never claims that.
- The archive-first CI lesson: a closed-late package leaves `.specsync/changes/`.
- No gate, bound, timeout or existing assertion is loosened.
