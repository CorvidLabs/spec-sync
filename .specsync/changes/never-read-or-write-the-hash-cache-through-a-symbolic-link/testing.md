---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: testing
---

# Testing

Every fixture is a temporary directory that holds both the project and the "outside" target, so no test touches a real file outside its own directory. The link tests are Unix-only (`#[cfg(unix)]`), as the product is built for Linux and macOS.

| Test | Proves | On `3bebe8b0` |
|---|---|---|
| `hash_cache::tests::save_refuses_a_linked_cache_file_and_leaves_its_target` | `save` errors, the link stays, the outside file keeps its bytes | fails: the outside file holds the cache JSON |
| `hash_cache::tests::save_creates_nothing_through_a_dangling_cache_link` | nothing is created at a dangling link's target | fails: the target is created |
| `hash_cache::tests::save_refuses_a_linked_cache_directory` | a linked `.specsync` gains no entry outside | fails: `hashes.json` appears outside |
| `hash_cache::tests::load_reads_no_cache_through_a_link` | a valid cache behind either link loads as empty | fails: the outside cache is loaded |
| `hash_cache::tests::a_directory_at_the_cache_path_is_refused` | not a regular file: `save` errors, `load` is empty, `refusal` names it | passes already for `save` and `load`; `refusal` is new |
| `hash_cache::tests::save_replaces_the_cache_by_rename` | a hard link to the old cache keeps the old bytes; no temporary file is left | fails: the hard link is written through |
| `check::check_never_writes_the_hash_cache_through_a_link` (integration) | plain, `--force`, `--strict` and JSON `check` keep their exit code, warn once, and leave the target unchanged; dangling and linked-directory cases | fails: the outside file is overwritten |
| `check::rehash_refuses_a_linked_hash_cache` (integration) | `rehash` exits 1, names the link, leaves the target unchanged | fails: exits 0 and overwrites it |

Then `cargo test hash_cache::`, `cargo test commands::check::tests::`, the full `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `specsync check --strict --require-coverage 100 --force`. The private fork runs no CI, so the full lane runs locally and its result is recorded as evidence.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-hash-cache-007 | `src/hash_cache.rs` tests `save_refuses_a_linked_cache_file_and_leaves_its_target`, `save_creates_nothing_through_a_dangling_cache_link`, `save_refuses_a_linked_cache_directory`, `load_reads_no_cache_through_a_link`, `a_directory_at_the_cache_path_is_refused` and `save_replaces_the_cache_by_rename`; `tests/integration/check.rs`: `rehash_refuses_a_linked_hash_cache` |
| REQ-cmd-check-018 | `tests/integration/check.rs`: `check_never_writes_the_hash_cache_through_a_link` |
