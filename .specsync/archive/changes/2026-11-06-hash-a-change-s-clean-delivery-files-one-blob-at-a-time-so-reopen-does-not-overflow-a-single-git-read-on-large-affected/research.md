---
change: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected
artifact: research
---

# Research

- **The refusal's source.** The message comes from `run_git_bounded` ("`git {}` output exceeds
  deterministic bounds"). Its only `cat-file --batch` caller is `git_blob_bytes_batch`
  (src/change/consider_accepted_evidence_anchor.rs), which `inspect_git_candidates` calls for clean blobs
  and `git_tree_snapshot` calls for legacy archive trees.
- **Why CHG-0026 hits it.** CHG-0026 is v1 with a 273-entry `acceptance_manifest`. Reopen therefore calls
  `acceptance_manifest_with_signed_owners`, which calls `acceptance_manifest_internal` with
  `AcceptanceDiscovery::Full` over its affected paths. A manifest-less change calls
  `acceptance_input_digest`. Both discover with `identify_clean_blobs = false`.
- **The per-object reader.** `git_blob_bytes` reads one object through `cat-file blob` with a
  `MAX_GIT_EVIDENCE_PAYLOAD_BYTES` cap. `memoized_configured_git` routes it to the shared batched reader
  while a fact scope is open.
- **Site measurement.** At site 3d858b83, `public/nevermore` holds 609 files and 390,503,241 bytes
  (372 MiB). The whole of `public` holds 10,896 files and 615 MiB.
