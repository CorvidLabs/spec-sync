# Lesson bundle — align-public-documentation-and-the-release-runbook-with-the-shipped-specsync-6-0-0

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Align public documentation and the release runbook with the shipped SpecSync 6.0.0
- **Kind**: Documentation
- **Paths**: README.md, MIGRATION.md, SECURITY.md, CONTRIBUTING.md, SCOPE.md, AGENTS.md, CHANGELOG.md, action.yml, fledge.toml, docs/RELEASING.md, docs/ADOPTING.md, docs/ci-confidence.md, site/src/content/docs/, examples/quickstart/README.md, examples/sdd-lifecycle/README.md, examples/sdd-concurrent-changes/README.md, examples/sdd-five-epics/README.md, vscode-extension/README.md, .github/workflows/ci.yml, .github/workflows/rc-assets.yml, .github/workflows/release.yml
- **Acceptance**: README, site docs, MIGRATION, ADOPTING, SECURITY, CONTRIBUTING, SCOPE, AGENTS, action metadata text, example READMEs and ci-confidence describe the shipped 6.0.0 binary and workflow with no candidate-window wording, retired CHG identities, rejected flags, or Windows support claims; the import caveat (#416) and the pre-adopt v1 guidance (#674) are present; docs/RELEASING.md gives a maintainer the exact qualify, promote, crates.io and Homebrew sequence; site lint, tests and build pass; release-version and runtime-pin validators pass; the example scripts still pass with the 6.0 binary

## Evidence

- Verification commit: `bd910f4fd3217cb59f573037bdc1f0aedded285c`
- Base commit: `38354c51ad3a4ba2408b8daf906dd1a9a0d5659f`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

PR #762 fixed the two-platform promotion guards and PR #764 fixed the init stamp, the standalone quickstart example and the 6.0.0 release notes. With both merged, main at 38354c51 is the intended 6.0.0 tree apart from one `check --fix` defect (#615) being fixed separately. A sweep of every public surface against the 6.0 binary, the release workflows and the 6.0.0 changelog found wording written for the candidate window or for the pre-6.0 lifecycle: candidate-selection install prose, `CHG-` identities in walkthroughs, `check` output samples in a 4.x format, an `init` description that still implied an interview, three-platform and Windows qualification claims, a release App and protected environment that were never provisioned, and a documented `import` command whose output fails `check` (#416). No maintainer runbook existed for the release lane, whose `promote` job has never executed.

## Constraints

Fix, do not restyle. Every corrected claim is checked against `specsync --help`, the source tree, `release.yml`, `ci.yml`, `trust.yml` and `fledge.toml`. Nothing here changes command grammar, runtime behaviour, workflow protections or canonical spec contracts; `action.yml` changes are description text only. No release date and no publication claim is made: crates.io and Homebrew serve 5.2.0 until published, and the final tree is frozen as the next immutable RC after this and the #615 fix merge.

## Implementation evidence

Site: Astro check 0 errors/warnings/hints, 23 site tests pass, 43-page build. Release-version and runtime-pin validators pass. Example scripts run with the 6.0 binary. Strict SpecSync validation and the mandatory pre-push gate are recorded on the PR.

## From the change's design.md

# Design

Keep existing structure, navigation and voice on every page. Replace claims with what the binary prints and what the workflows do; add caveats beside the command they qualify rather than in new sections. The runbook is a standalone operator document under docs/, command-first, explicit about steps that have never executed.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
