---
change: advance-the-published-specsync-6-1-2-action-default-and-archive-gate-examples-after-platform-consumer-checks
artifact: design
---

# Design

Advance only `PUBLISHED_ACTION_DEFAULT`, the `action.yml` version input default and description, the Action docs default row, and current README and CHANGELOG prose from 6.1.1 to 6.1.2. Move archive-gate examples from `@main` to `@v6.1.2` with `version: '6.1.2'`; with the published default at 6.1.2 the validator's archive-gate `@main` allowance ends by itself. The composite `runs` steps, downloader, checksum and runtime behavior, and the explicit CI and Trust candidate mirrors stay unchanged.

The github spec Purpose is updated through one MODIFIED SPEC SECTION Purpose delta; REQ-github-002 stays byte-identical. Immutable v6.1.2 keeps default 6.1.1, so docs tell `@v6.1.2` users to pass `version: '6.1.2'`. Floating v6 advances only after omitted-input Linux and macOS consumers pass at the merged wrapper commit.
