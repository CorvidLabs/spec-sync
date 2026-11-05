---
change: prepare-specsync-6-2-0-package-metadata-and-release-handoff
artifact: tasks
---

# Tasks

- [x] Inspect package surfaces, release workflows and the 6.1.2 preparation.
- [x] Bump Cargo.toml, Cargo.lock, the CI packaged Action consumer and the Trust candidate to 6.2.0.
- [x] Update release validator fixtures to the 6.2.0 candidate.
- [x] Write the 6.2.0 CHANGELOG section and docs/releases/6.2.0.md.
- [x] Move documentation examples to @v6.2.0 and refresh current-install guidance to published 6.1.2.
- [x] Run the three release validators.

Human gates and post-verification steps (not tasks): merging main after #842, #845 and
#846 land, or removing their entries; native verification with
`change check --commit`; the human implementation review; finalization on this pull
request before merge; qualification of the immutable candidate on the required hosted
platforms; and publishing and independently verifying distributions, each only after
separate authorization (see docs/releases/6.2.0.md).
