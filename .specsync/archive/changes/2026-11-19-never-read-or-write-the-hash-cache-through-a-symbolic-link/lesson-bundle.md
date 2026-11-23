# Lesson bundle: never-read-or-write-the-hash-cache-through-a-symbolic-link

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Never read or write the hash cache through a symbolic link
- **Kind**: BugFix
- **Specs**: hash_cache, cmd_check
- **Paths**: src/hash_cache.rs, src/commands/check.rs, tests/integration.rs, tests/integration/, specs/hash_cache/, specs/cmd_check/, hi/check.md, INTENT.md, CHANGELOG.md, site/src/content/docs/cli.md, docs/HLD.md
- **Acceptance**: With .specsync/hashes.json, or .specsync itself, as a symbolic link to a path outside the repository, specsync check and specsync rehash leave the link's target byte for byte unchanged and create nothing through it; check keeps its verdict and exit code and prints one stderr warning that the hash cache was not saved; rehash exits 1 and names the link; load returns an empty cache without reading through a link; a regular cache is replaced by a temp file in .specsync renamed over it; the new tests fail on 3bebe8b0 and pass with the fix

## Evidence

- Verification commit: `b60cac3895cb615daf9ce8a0426d398548dc2599`
- Base commit: `3bebe8b03094b2079630801584b56f94a1e460c5`
- Verified by: `specsync check --spec cmd_check --spec hash_cache`

## From the change's context.md

# Context

Reported privately as GHSA-7q5v-8jcc-rx54 and handled under `SECURITY.md`. `HashCache::save` wrote `.specsync/hashes.json` with `fs::write`, which follows a symbolic link. A repository can commit that path as a link, since Git stores links and a `.gitignore` entry does not stop a forced add. Anyone who then clones it and runs `specsync check` (which saves the cache after a run with no errors) or `specsync rehash` overwrites the file the link names, outside the repository, with the cache JSON. The repository chooses the JSON's keys: they are its spec and source paths. A dangling link creates the file. `.specsync` itself can be the link, which puts `hashes.json` into a directory outside the repository. `HashCache::load` read through the same links.

Human intent: CHECK-1.e, "Running the check on a repository somebody else wrote never changes a file outside that repository."

What a session picking this up needs to know:

- The work stays in the advisory's private fork until the advisory is published. Nothing about it goes to the public repository, an issue or a comment before then.
- The scope the human approved: load reads a linked or non-regular cache as empty; save refuses when `.specsync` or `.specsync/hashes.json` is a symbolic link or not a regular file, and otherwise writes a new file in `.specsync` and renames it; `check` skips the cache with one stderr warning and keeps its verdict and exit code; `rehash` exits 1.
- Ruled out: replacing a linked `hashes.json` with a regular file. A rename would be safe, but it hides a planted link and changes a tracked file without saying so.
- Not in this change: other writers under `.specsync`, and the files a spec's `files:` list makes the cache hash. A separate private sweep covers them.

## From the change's design.md

# Design

All of it is in `src/hash_cache.rs`, plus the warning in `src/commands/check.rs`.

- **One way into the cache directory.** A private helper opens the project root as a `cap_std::fs::Dir`, looks at `.specsync` with `symlink_metadata`, and opens it with `cap_primitives::fs::open_dir_nofollow`. A link or a non-directory is refused with an error that names `.specsync` and what it is. A missing directory is created for a save and reads as "no cache" for a load.
- **`save`.** Refuses when `.specsync/hashes.json` exists and is not a regular file. Otherwise it creates `.hashes.json.<pid>.<n>.tmp` in the directory handle with an exclusive create, writes the JSON, and renames it over `hashes.json` in the same handle. A failed write or rename removes the temporary file. The file keeps the default creation mode, as before.
- **`load`.** Opens `hashes.json` in the directory handle with no-follow and non-blocking flags, checks on the open handle that it is a regular file, then reads and parses it. Every failure is an empty cache, as before.
- **`refusal`.** `HashCache::refusal(root) -> Option<String>` returns the reason a location is refused. It is the text `check` prints.
- **`check`.** Asks for the refusal once before loading. With one, it prints `warning: hash cache not used: <reason>` on stderr and does not save. Its stdout, verdict and exit code are those of a run with no cache.
- **`rehash`.** No source change. `save` returns the refusal and the command prints it and exits 1, as it does for any save error.

Alternatives not taken:

- Path checks with `fs::symlink_metadata` followed by `tempfile` in the path: correct for a committed link, but a path component swapped between the check and the write is followed. The directory handle costs a few lines and closes that.
- Opening `hashes.json` for writing in place with no-follow: no rename, so a hard link to the file is still written through, and a reader can see a half-written cache.
- A warning for every failed save: more honest, but a change for read-only checkouts that nobody asked for here.

## From the change's testing.md

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

## Where these lessons go

- `specs/hash_cache/hash_cache.3md`
- `specs/cmd_check/cmd_check.3md`
