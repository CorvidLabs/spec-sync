---
change: prepare-specsync-6-4-2-package-metadata-and-release-handoff
artifact: context
---

# Context

The latest GitHub release is v6.4.1. The maintainer requested a bug-fix release and the integration of existing correctness PRs. The release comparison starts at v6.4.1. Record fixes under the new version once they reach main. The three fixes already on main synchronize change state, surface frozen Change Log deltas during audit, and update the url lockfile entry.

At preparation time, main still resolves cookie 0.18.1. PR #1148 addresses the current advisory. A release candidate needs the dependency audit and the hosted checks to pass on the selected main revision.

Final reconcile, 2026-11-24. The release contents are the 21 pull requests merged since v6.4.1, through #1149. Main's nine new Unreleased entries moved into the 6.4.2 section, and #1105 gained the entry #1149 did not add, so the section holds 32 entries and Unreleased is empty. The handoff has one line per pull request. Cargo.lock differs from main only in the package version, and the hosted dependency audit passes on main at `6c5dc5b2`.

One thing dogfooding showed and this change leaves alone: `specsync stale` reports `cmd_change` six commits behind its source on main, where it reported nothing at v6.4.1. No gate runs that command.
