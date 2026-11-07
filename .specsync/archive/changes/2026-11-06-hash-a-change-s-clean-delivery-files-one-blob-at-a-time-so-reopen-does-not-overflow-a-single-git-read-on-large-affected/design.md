---
change: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected
artifact: design
---

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
