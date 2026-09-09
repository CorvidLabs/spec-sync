---
change: align-public-documentation-and-the-release-runbook-with-the-shipped-specsync-6-0-0
artifact: context
---

# Context

PR #762 fixed the two-platform promotion guards and PR #764 fixed the init stamp, the standalone quickstart example and the 6.0.0 release notes. With both merged, main at 38354c51 is the intended 6.0.0 tree apart from one `check --fix` defect (#615) being fixed separately. A sweep of every public surface against the 6.0 binary, the release workflows and the 6.0.0 changelog found wording written for the candidate window or for the pre-6.0 lifecycle: candidate-selection install prose, `CHG-` identities in walkthroughs, `check` output samples in a 4.x format, an `init` description that still implied an interview, three-platform and Windows qualification claims, a release App and protected environment that were never provisioned, and a documented `import` command whose output fails `check` (#416). No maintainer runbook existed for the release lane, whose `promote` job has never executed.

## Constraints

Fix, do not restyle. Every corrected claim is checked against `specsync --help`, the source tree, `release.yml`, `ci.yml`, `trust.yml` and `fledge.toml`. Nothing here changes command grammar, runtime behaviour, workflow protections or canonical spec contracts; `action.yml` changes are description text only. No release date and no publication claim is made: crates.io and Homebrew serve 5.2.0 until published, and the final tree is frozen as the next immutable RC after this and the #615 fix merge.

## Implementation evidence

Site: Astro check 0 errors/warnings/hints, 23 site tests pass, 43-page build. Release-version and runtime-pin validators pass. Example scripts run with the 6.0 binary. Strict SpecSync validation and the mandatory pre-push gate are recorded on the PR.
