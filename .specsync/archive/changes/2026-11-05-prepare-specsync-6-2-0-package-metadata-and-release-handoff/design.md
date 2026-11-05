---
change: prepare-specsync-6-2-0-package-metadata-and-release-handoff
artifact: design
---

# Design

Keep the runtime and public API byte-identical apart from the declared package version. Retain the published Action installer default (6.1.2) and existing supported publication destinations. Because the published default is already past 6.1.1, the archive-gate `@main` allowance has ended, so archive-gate examples pin the candidate like every other example. A later release uses the reviewed and merged candidate.
