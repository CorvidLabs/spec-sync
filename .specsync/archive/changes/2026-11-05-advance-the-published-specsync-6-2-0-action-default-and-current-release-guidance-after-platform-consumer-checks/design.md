---
change: advance-the-published-specsync-6-2-0-action-default-and-current-release-guidance-after-platform-consumer-checks
artifact: design
---

# Design

Advance only `PUBLISHED_ACTION_DEFAULT`, the `action.yml` version input default and description, the Action docs default row, and current README, SECURITY, MIGRATION, ADOPTING, site and CHANGELOG prose from 6.1.2 to 6.2.0. Archive-gate examples already pin `@v6.2.0` with `version: '6.2.0'`. The composite `runs` steps, downloader, checksum and runtime behavior, and the explicit CI and Trust candidate mirrors stay unchanged. Validator logic is unchanged; the release-validator tests model the candidate window independently of the current default, so they need no fixture edit unless a run shows otherwise.

The github spec Purpose is updated through one MODIFIED SPEC SECTION Purpose delta; REQ-github-002 stays byte-identical. Immutable v6.2.0 keeps default 6.1.2, so docs tell `@v6.2.0` users to pass `version: '6.2.0'`. Floating v6 advances only after omitted-input Linux and macOS consumers pass at the merged wrapper commit.
