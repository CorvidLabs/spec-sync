---
change: CHG-0150-the-release-lane-must-not-gate-on-a-check-no-workflow-produces
artifact: requirements
---

# Requirements

This change is `--no-spec-change`: `.github/workflows/release.yml` is CI configuration with no
owning spec module, so it adds no `REQ-` block to the living tree (precedent CHG-0014).

The obligations it does carry, and where each is evidenced in `testing.md`:

1. **`release.yml` waits on no check-run name that has no producer.** Evidenced by the
   producer/consumer check, which fails against the file at `origin/main` and passes against
   this one, with two real producer/consumer pairs green in both runs so the pass is not
   vacuous.

2. **An RC tag reaches `qualify`.** `validate` no longer contains a wait that cannot be
   satisfied. The three checks it keeps — tag version equals the `Cargo.toml` package version,
   the checkout is the resolved candidate, the candidate is an ancestor of `origin/main` — are
   unchanged and still fail closed.

3. **The lane is executable without creating a tag.** `dry_run=true` resolves `mode=dry-run`,
   which reaches only `resolve` and `validate`; every job that tags or publishes is gated on
   `qualify` or `promote` and skips. An unrecognized `dry_run` value is rejected rather than
   defaulted, so a typo cannot become a promotion.

## Deliberately unchanged

The `qualify` and `promote` job graphs, their guards, and every check outside the deleted
binding block. This change narrows what the release lane *verifies*; it does not change what it
*does* in either existing mode.

## Property removed, recorded here rather than implied away

The deleted block asserted that the bound pull request was merged, that the archive is
`workflow_version=2`, that its finalization evidence validates, and that the PR has an exact
implementation head. Those assertions are unreachable today because their input is never
produced, but they were real. After this change the archive-to-merge-commit binding is enforced
solely by SpecSync itself, in `change ship` and archive validation — which is where #499 placed
the lifecycle when it deleted the producer.
