---
kind: evidence
id: 2026-11-19-on-base-3bebe8b0-with-only-the-new-tests-added-5-of-6-new-hash
aliases: []
claim: "On base 3bebe8b0 with only the new tests added: 5 of 6 new hash_cache unit tests and both new integration tests fail. check and rehash overwrite the outside file a linked .specsync/hashes.json names with the cache JSON, create a dangling link's target, write hashes.json into a linked .specsync, load a cache through either link, and write in place through a hard link."
result: failed
evidence_kind: product-test
method: "cargo test --locked --bin specsync -- hash_cache:: ; cargo test --locked --test integration -- check_never_writes_the_hash_cache_through_a_link rehash_refuses_a_linked_hash_cache (rustc 1.89.0)"
revision: 3bebe8b03094b2079630801584b56f94a1e460c5
content: 40e1ce5714a4a00d48453d0d4d965a07f38808ab90c5364b5377f20e02e41ab0
environment: local-macos-arm64
criteria: [CHECK-1.e]
requirements: []
contracts: []
limitations:
  - "Unit result: 20 passed, 5 failed. Integration result: 0 passed, 2 failed. a_directory_at_the_cache_path_is_refused already passes on base. Shows REQ-hash-cache-007 and REQ-cmd-check-018; not linked with --requirement because Chough looks for specs/hash-cache/ and the module is hash_cache."
observed: 2026-11-19T04:31:02Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On base 3bebe8b0 with only the new tests added: 5 of 6 new hash_cache unit tests and both new integration tests fail. check and rehash overwrite the outside file a linked .specsync/hashes.json names with the cache JSON, create a dangling link's target, write hashes.json into a linked .specsync, load a cache through either link, and write in place through a hard link.**

Recorded as failed in local-macos-arm64 at revision 3bebe8b03094, observed 2026-11-19T04:31:02Z.

## Key failing lines

- check [] wrote the hash cache through the link, over a file outside the project; rehash wrote the hash cache through the link, over a file outside the project; the cache was written into a directory outside the project; the cache was created at the link's target, outside the project; a cache was read through a linked cache file; the old cache file was written in place, through its other name
