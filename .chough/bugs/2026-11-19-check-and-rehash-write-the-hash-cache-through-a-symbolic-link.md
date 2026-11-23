---
kind: bug
id: 2026-11-19-check-and-rehash-write-the-hash-cache-through-a-symbolic-link
aliases: []
title: check and rehash write the hash cache through a symbolic link
status: verified
classification: product-defect
scope: blocking
impact: Anyone who clones a repository that commits the link and runs check or rehash there loses the file the link names, with their own permissions. Present since the hash cache was added
criteria: [CHECK-1.e]
requirements: []
cases: []
evidence:
  - 2026-11-19-hash-cache-symlink-write/2026-11-19-on-base-3bebe8b0-with-only-the-new-tests-added-5-of-6-new-hash
  - 2026-11-19-hash-cache-symlink-write/2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both
receipts: []
fixture: Temporary project beside an outside directory in src/hash_cache.rs tests and tests/integration/check.rs; every link points inside the fixture
environment: local-macos-arm64
revision: 3bebe8b03094b2079630801584b56f94a1e460c5
found_by: claude-code (hc t5r) (private security report GHSA-7q5v-8jcc-rx54)
actor:
  - agent: claude-code
    terminal: t5r
related: []
created: 2026-11-19T04:31:11Z
updated: 2026-11-19T04:48:13Z
---
# check and rehash write the hash cache through a symbolic link

**What happened:** A file outside the repository is overwritten with the hash cache JSON, or created by it

Actual: HashCache::save calls fs::write, which follows the link; the target is replaced by the cache JSON, whose keys are the repository's spec and source paths. A dangling link creates its target, a linked .specsync receives hashes.json, and load reads through the same links

**Steps:** Trigger: specsync check (with no validation errors) or specsync rehash in a repository whose .specsync/hashes.json, or .specsync itself, is a symbolic link
1. Commit .specsync/hashes.json as a symbolic link to a path outside the repository
2. Clone the repository and run specsync check
3. Read the file the link names: it holds the cache JSON

**Expected:** The cache is never read or written through a link: check says once that the cache was not used and keeps its verdict and exit code, rehash exits 1, and the link's target is unchanged

**Why it matters:** Anyone who clones a repository that commits the link and runs check or rehash there loses the file the link names, with their own permissions. Present since the hash cache was added

**Fix:** Verified: 2026-11-19-hash-cache-symlink-write/2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both passed at 3bebe8b03094b2079630801584b56f94a1e460c5.

## Log

- 2026-11-19T04:31:11Z · Reported (private security report GHSA-7q5v-8jcc-rx54); evidence: 2026-11-19-hash-cache-symlink-write/2026-11-19-on-base-3bebe8b0-with-only-the-new-tests-added-5-of-6-new-hash · by claude-code (hc t5r)
- 2026-11-19T04:48:13Z · status: open -> verified; evidence: 2026-11-19-hash-cache-symlink-write/2026-11-19-with-the-fix-the-6-new-hash-cache-unit-tests-and-both. Fixed by the native change never-read-or-write-the-hash-cache-through-a-symbolic-link: the cache is read and written through a no-follow .specsync directory handle and saved by rename; a linked or non-regular location is refused. The same tests that failed on 3bebe8b0 pass. · by claude-code (hc t5r)
