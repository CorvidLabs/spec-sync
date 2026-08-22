---
change: a-release-candidate-must-be-installable-without-release-grade-provenance
artifact: context
---

# Context

`v6.0.0-rc.1` was tagged, correctly, as an annotated pre-release on the right commit. It carries
zero assets, because the release lane refuses at its first job and everything downstream skips.
Every consumer of the packaged action still 404s.

The lane had never completed a run. #635 was the first reason found; #668 was the second. An
audit of the six jobs that had still never executed found four more, and the audit also
corrected #668's fix — `fetch-tags: true` is a **no-op** on this code path, so the fix that
merged does not work. That correction is here.

The wider point this change acts on: a final tag and a release candidate are different promises.
A final tag deserves rulesets, a protected environment and a signing identity, and `release.yml`
should keep all of it. A candidate's entire job is to be installed by people who have agreed to
test it — and gating it behind release-grade provenance means it cannot be installed at all,
which is exactly what happened. `rc-assets.yml` gives a candidate a path that does not depend on
provenance the candidate does not need, while refusing to touch anything that is not a
pre-release.

Three blockers remain that are repository configuration rather than code: three tag rulesets, a
protected `release` environment, and a `SPECSYNC_RELEASE_APP_ID` variable. They gate the final
lane only, and are deliberately not in scope here.
