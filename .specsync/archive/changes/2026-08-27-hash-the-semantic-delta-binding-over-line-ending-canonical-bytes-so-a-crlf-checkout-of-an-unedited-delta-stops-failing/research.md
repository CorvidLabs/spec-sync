---
change: hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing
artifact: research
---

# Research

## Measured: the archive does not move

All 198 archived `approvals.json` under `.specsync/archive/changes/` were recomputed with the
framed preimage the code uses — `frame("domain", "specsync.approved-delta.v1")`,
`frame("module", <module>)`, `frame("body", <bytes>)`, SHA-256, lowercase hex — once over the raw
delta bytes and once over the CRLF-folded bytes.

| Measurement | Value |
|---|---|
| Ledgers scanned | 198 |
| Ledgers carrying `approved_delta_digests` | 8 |
| Module digest records recomputed | 25 |
| Records where raw and normalized digests differ | **0** |
| Archived delta files containing a CRLF pair | 0 |
| Archived delta files containing a lone carriage return | 0 |

The recomputation is self-validating: 23 of the 25 recorded digests are reproduced exactly by the
script, which is what establishes the script computes the same preimage the binary does.

The other 2 match neither recomputation, and the cause is recorded history rather than this
change. Both live in `2026-08-25-bind-semantic-delta-bodies-to-the-approval-that-signed-them`
(#711), whose ledger holds three definition approvals:

```
0 definition 1787611212 {'change': 'b30dfb39…'}
1 definition 1787629772 {'change': 'b68a45d5…'}
2 definition 1787661974 {'change': 'e02fc3e7…'}   <- effective, matches the archived delta
```

#711's own PR reported this: its delta was edited by a rebase after approval, the new gate caught
it, and it was re-approved. `b30dfb39…` is even the digest quoted in #709's exposure measurement.
Both stale records differ identically under the raw and the normalized preimage, so this change
does not move them.

## Measured: nothing else hashes filesystem text

- `FramedDigest::new` appears at 30 sites, every one in `src/change.rs`.
- `digest.frame(b"body", …)` appears exactly once, in `delta_body_digests`.
- `read_bounded_change_text` has nine call sites. Eight feed `parse_delta` or
  `artifact_content_is_incomplete`; only `delta_body_digests` hashed the text it read.

## Measured: the definition digest reads Git blobs, not the working tree

`definition_artifact_snapshot` → `git_regular_file_evidence` → `inspect_git_candidates` →
`capture_git_candidate`. For a path that is tracked and clean, the payload is
`git_blob_bytes(root, object)`, i.e. `git cat-file blob`. Git stores blobs LF-normalized whatever
`core.autocrlf` or a `text` / `eol` attribute does to the working tree, and `git diff-files`
applies the same conversion, so an eol-converted checkout is still *clean* and still hashes the LF
blob. The #730 scenario cannot move `definition_digest`, `execution_digest`,
`project_input_digest`, or the acceptance manifest.

`validate_canonical_git_attributes` reinforces the reading: it refuses `filter`,
`working-tree-encoding` and `ident` — the attributes that would make the blob and the working tree
disagree in ways Git cannot undo — and deliberately does not list `text` / `eol`, because those
are the conversion the blob already normalizes away.

The one fallback to working-tree bytes is `capture_working_candidate`, taken when a path is dirty
or untracked. The module already knows this: `append_portable_definition_approval_v501` compares
each snapshot entry's working-tree bytes against its canonical blob and refuses with
"SpecSync 5.0.1 portable projection requires canonical working-tree bytes for `{relative}`; use a
canonical LF release checkout", covered by
`portable_projection_rejects_clean_crlf_smudging_before_ledger_mutation`.

## Read, and load-bearing

- **#709** — both remedies, and the argument that normalizing forfeits no security property
  because `parse_delta` already discards the carriage return.
- **#715** — landed the `.gitattributes` half, and explicitly recorded that the digest half was
  still owed. Also the source of the `Cow`-guarded normalization shape and of the decision to
  preserve a lone carriage return.
- **#711** — the gate that misfires, its "absent evidence is unknown" reading, and its own
  disclosure of this exposure.
- **#719 / #727** — the monotonicity fix on the same binding; its tests are the neighbours these
  sit beside.
