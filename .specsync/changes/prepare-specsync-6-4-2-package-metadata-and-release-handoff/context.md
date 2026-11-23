---
change: prepare-specsync-6-4-2-package-metadata-and-release-handoff
artifact: context
---

# Context

The latest GitHub release is v6.4.1. The maintainer requested a bug-fix release and the integration of existing correctness PRs. The release comparison starts at v6.4.1. Record fixes under the new version once they reach main. The three fixes already on main synchronize change state, surface frozen Change Log deltas during audit, and update the url lockfile entry.

At preparation time, main still resolves cookie 0.18.1. PR #1148 addresses the current advisory. A release candidate needs the dependency audit and the hosted checks to pass on the selected main revision.
