---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: context
---

# Context

SpecSync `v5.1.0` was published from main commit `fc6e70b` on 2026-07-15. PR #387
subsequently landed the fail-closed lifecycle evidence, performance, security, and Windows
portability corrections as main commit `4652ca1`. Because the published tag is immutable, those
corrections require a patch release rather than a rewritten 5.1.0 artifact.

The current distribution surfaces are inconsistent: crates.io and GitHub Releases expose 5.1.0,
the composite Action defaults to 5.0.0, the README's immutable example pins 5.0.0, the Action docs
report a 5.0.0 default, and the Homebrew formula remains at 5.0.1. The documentation also
recommends `CorvidLabs/spec-sync@v5`, but no `v5` branch or tag currently exists.

CHG-0043 through CHG-0047 are accepted, squash-integrated, strict-valid on main, and archived on
this branch. CHG-0048 owns the SpecSync repository release candidate. Crates.io publication,
GitHub tags/releases, the mutable compatibility ref, Homebrew, and the Trust rollout are external
delivery actions that occur only after this repository's candidate is accepted and integrated.

The first post-merge main run exposed a separate infrastructure fragility: Pages plus the site and
VS Code CI jobs all invoked `oven-sh/setup-bun` without an exact version. During a GitHub API 503,
the action's live lookup of `oven-sh/bun` tags failed before any repository command ran. CHG-0048
therefore also pins the locally supported Bun 1.3.14 runtime consistently across those jobs.
