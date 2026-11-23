---
spec: rehash.spec.md
---

## Tasks

- [x] Implement full hash-cache regeneration command
- [x] Ensure cache save failures return a non-zero exit
- [x] Save the cache in the local state folder and print the path (REQ-rehash-003). Evidence: `rehash_never_writes_the_hash_cache_through_a_link`.

## Gaps

- Save-failure behavior is specified but difficult to exercise portably across platforms.

## Post-5.0 Test Debt

- [ ] Add CLI-level test for save failure on read-only `.specsync` directory

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
