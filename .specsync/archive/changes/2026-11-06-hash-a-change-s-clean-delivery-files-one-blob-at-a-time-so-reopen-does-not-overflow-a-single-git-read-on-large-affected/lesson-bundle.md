# Lesson bundle: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Hash a change's clean delivery files one blob at a time so reopen does not overflow a single Git read on large affected paths
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/, src/change_tests/, tests/, specs/change/, CHANGELOG.md
- **Acceptance**: SHIP-6 (REQ-change-144): reopen of a stale accepted change whose clean affected files exceed one git cat-file --batch read, with or without an acceptance manifest, no longer fails with "`git cat-file --batch` output exceeds deterministic bounds"; a test-only seam shrinks the cap so the test runs on small fixtures.
- **Acceptance**: The acceptance manifest and the acceptance input digest are byte-identical to the ones the whole-scope read produced, for clean, modified, symlinked, deleted and executable files, an LFS pointer and the sequence ledger, so every recorded manifest and digest still matches.
- **Acceptance**: Each clean blob is still bounded by MAX_GIT_EVIDENCE_PAYLOAD_BYTES, and a blob over it is refused by name; the path-count bound and the Git read deadline are unchanged, and every other caller of inspect_git_candidates keeps both of its caps.
- **Acceptance**: The accepted-input validation behind change status and the archive preflight builds that evidence the same way.
- **Acceptance**: Failing tests first: each new test fails on main f7180c34 and passes after the change; no gate, timeout or assertion is loosened.

## Evidence

- Verification commit: `481f588d9796f14d27d9a598d2dc876f76df9bfb`
- Base commit: `f7180c3481814c7b05671fb413b1ffec5de829e2`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#976: `specsync change reopen` refuses before it checks anything when a change's affected paths hold more
clean file bytes than one in-memory Git read allows. The refusal is "`git cat-file --batch` output exceeds
deterministic bounds". On CorvidLabs/site, reopen of CHG-0026 (site PR #232) fails this way in 1.8 s on a
throwaway clone of site origin/main 3d858b83, whose `public/nevermore` holds 609 files and 372 MiB.

Intent: SHIP-6 ("When closed evidence goes stale there is an audited way to reopen it"). No new criterion;
Leif approved the plan through orc t18 ("Capture all + approve design", 2026-11-06), and the orc claimed
REQ-change-144. The late close (#833, REQ-change-143) is a separate PR and goes first.

## From the change's design.md

# Design

**Today.** `acceptance_manifest_internal` and `acceptance_input_digest` call
`stable_discovered_evidence(…, identify_clean_blobs = false)`. `inspect_git_candidates` then fetches every
clean blob in one `git cat-file --batch` call (`git_blob_bytes_batch`) into memory. That read is capped at
`MAX_GIT_EVIDENCE_PAYLOAD_BYTES + MAX_GIT_EVIDENCE_PATH_BYTES` (320 MiB), and the captured total is capped
again at `MAX_GIT_EVIDENCE_PAYLOAD_BYTES` (256 MiB). The bytes are then only hashed: sha256 per manifest
entry, or the framed acceptance digest.

**After.**

1. Both builders discover with `identify_clean_blobs = true`, which already exists: clean regular files are
   named by object id, while LFS pointers and symlinks are still materialized.
2. For each entry named by object id, the builder reads that one blob with `git_blob_bytes`. That reader
   applies the existing per-object `MAX_GIT_EVIDENCE_PAYLOAD_BYTES` cap and is served by the shared batched
   reader (REQ-git-utils-010).
3. The bytes are hashed into the entry or the digest and dropped before the next blob.
4. Modified, missing, gitlink and override entries are handled as today.

**Bound change, stated plainly.**

- These two builders no longer have an all-at-once cap on the total bytes of clean files. Memory stays
  bounded by one blob, and each blob keeps its 256 MiB cap.
- The 100,000-path cap and the Git read deadline still apply.
- Every other caller of `inspect_git_candidates` keeps both caps unchanged.

**Equality.** The blob bytes are the same object bytes that `git_blob_bytes_batch` returned, hashed in the
same order with the same framing. Recorded manifests and digests therefore keep matching.

**Test seam.** A `#[cfg(test)]` thread-local shrinks the batch and per-object caps, so the failing tests
can use small fixtures. Production values do not change.

## From the change's testing.md

# Testing

Every test below fails on main f7180c34 first, and that is recorded.

1. **Reopen past the cap.** With the batch cap shrunk by the test seam, reopen of a stale accepted change
   whose clean affected files exceed the cap no longer fails with "`git cat-file --batch` output exceeds
   deterministic bounds". This is checked both for a change with an acceptance manifest and for one without.
2. **Byte equality.** The streamed manifest and digest equal the batched ones on a fixture with clean,
   modified, symlinked, deleted and executable files, an LFS pointer and the sequence ledger.
3. **Per-blob cap.** A single blob over the shrunk per-object cap still refuses, and the refusal names the
   blob.
4. **Validation reads.** The accepted-input validation behind status and the archive preflight reads the
   same way, so an accepted change past the cap is judged on its evidence, not refused on the read.

One-off local check, not in CI: on a throwaway clone of site origin/main 3d858b83, `specsync change reopen
CHG-0026-… --actor dry-run` gets past the read. The next refusal is reported, not fixed.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-144 | tests 1 to 4 |

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| Reopen past the cap | test 1 |
| Byte-identical evidence | test 2 |
| Per-blob bound kept | test 3 |
| Validation reads | test 4 |
| Failing tests first | recorded failures on f7180c34 |

## Verification commands

- `fledge lanes run verify` on the pinned rust 1.89 (rustup proxy first on PATH)
- `specsync change check <id> --commit`
- `specsync change audit --strict`
- `fledge lanes run pre-push`

## Where these lessons go

- `specs/change/change.3md`
