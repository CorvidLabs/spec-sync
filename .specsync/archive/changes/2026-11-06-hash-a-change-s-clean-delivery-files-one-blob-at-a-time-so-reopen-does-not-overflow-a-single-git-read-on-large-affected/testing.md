---
change: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected
artifact: testing
---

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
