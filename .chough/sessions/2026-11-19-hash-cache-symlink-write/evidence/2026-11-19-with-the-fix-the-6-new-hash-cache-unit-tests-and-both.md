---
kind: evidence
id: 2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both
aliases: []
claim: "With the fix, the 6 new hash_cache unit tests and both integration tests pass: check (plain, --force, --strict, JSON, --spec auth JSON) and rehash leave a linked cache target byte for byte unchanged, create nothing through a dangling link or a linked .specsync, and check warns once with unchanged stdout and exit code."
result: passed
evidence_kind: product-test
method: "cargo test --locked (unit binary: 3079 passed, 4 failed, 2 ignored; the 4 are changelog::tests::test_generate_changelog_*, which fail on this machine's global git tag.gpgsign and pass with the isolated git config); cargo test --locked --test integration -- check_never_writes_the_hash_cache_through_a_link rehash_refuses_a_linked_hash_cache (2 passed); rustc 1.89.0"
revision: 3bebe8b03094b2079630801584b56f94a1e460c5
content: cba0c29436883f48d0b97280911b27f4a76fe37572df6382a92a6be54fea1345
environment: local-macos-arm64
criteria: [CHECK-1.e]
requirements: []
contracts: []
limitations:
  - Shows REQ-hash-cache-007 and REQ-cmd-check-018; not linked with --requirement because Chough looks for specs/hash-cache/ and the module is hash_cache. macOS arm64 only; Ubuntu not run. The full integration suite, release build and strict spec check are recorded separately.
observed: 2026-11-19T04:48:07Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the fix, the 6 new hash_cache unit tests and both integration tests pass: check (plain, --force, --strict, JSON, --spec auth JSON) and rehash leave a linked cache target byte for byte unchanged, create nothing through a dangling link or a linked .specsync, and check warns once with unchanged stdout and exit code.**

Recorded as passed in local-macos-arm64 at revision 3bebe8b03094, observed 2026-11-19T04:48:07Z.
