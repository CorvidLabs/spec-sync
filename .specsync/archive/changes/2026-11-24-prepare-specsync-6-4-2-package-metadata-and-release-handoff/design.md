---
change: prepare-specsync-6-4-2-package-metadata-and-release-handoff
artifact: design
---

# Design

Use the release-preparation surfaces established by #1138. Cargo, the CI action consumer, the Trust workflow, the candidate test fixtures and the documented explicit Action examples reference 6.4.2. The release validator uses 6.4.1 for published defaults until the release is available.

The handoff identifies the source revision and release evidence as they become available. Link each user-visible fix to its merged PR. The documented release workflow qualifies the immutable candidate on Linux and macOS before promotion.
