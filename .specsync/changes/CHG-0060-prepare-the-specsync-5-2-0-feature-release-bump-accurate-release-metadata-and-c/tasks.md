---
change: CHG-0060-prepare-the-specsync-5-2-0-feature-release-bump-accurate-release-metadata-and-c
artifact: tasks
---

# Tasks

- [x] Bump Cargo.toml/Cargo.lock to 5.2.0 and regenerate the lockfile.
- [x] Write the CHANGELOG.md 5.2.0 section covering all five shipped features and their issues.
- [x] Synchronize action.yml, ci.yml/trust.yml consumer pins, README, and site docs to 5.2.0.
- [x] Add REQ-github-004 with the 5.2.0 Action promotion contract via semantic delta.
- [x] Run format, lint, unit/integration tests, release validators, strict specs, and Trust.
- [x] Open the draft release PR, clear review, accept, and merge.
