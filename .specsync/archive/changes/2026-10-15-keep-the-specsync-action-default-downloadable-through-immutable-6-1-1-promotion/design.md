---
change: keep-the-specsync-action-default-downloadable-through-immutable-6-1-1-promotion
artifact: design
---

# Design

Pin the declared published default independently of the package candidate. Change the Action omitted default and documented default to6.1.0. Keep explicit CI/Trust6.1.1 localmirror inputs and all release/download/checksum/runtime guards byte-identical. Add twelve actual production-validator subprocess tests in the existing wired suite. Immutablev6.1.1 will retain default6.1.0; explicitversion6.1.1 selects its new binary. ConditionalPhaseB changes only the later wrapper/default declaration/current docs once its actual activation gates pass. No dynamic release selector or dependency is added.
