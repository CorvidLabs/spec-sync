---
change: a-release-candidate-must-be-installable-without-release-grade-provenance
artifact: plan
---

# Plan

Four repairs to `release.yml`:

1. `resolve` — drop the no-op `fetch-tags: true` and fetch the tag object explicitly on the push
   path, as the dispatch path already does.
2. `qualify` and `release` — `shell: python` becomes `shell: python3 {0}` at both sites.
3. `authorize-release` — find the workflow run by identity rather than by parsing a display URL
   GitHub rewrites.
4. `validate` — pin the toolchain rather than letting the candidate name it.

Then `rc-assets.yml`: build the same six targets under the same names and attach them to an
existing pre-release, guarded so it can only ever touch one.
