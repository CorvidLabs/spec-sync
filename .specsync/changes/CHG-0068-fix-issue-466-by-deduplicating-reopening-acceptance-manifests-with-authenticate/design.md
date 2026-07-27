---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: design
---

# Design

## Persisted Representation

New reopening events use schema version 2. Their prior verification keeps every existing field
except the large `acceptance_manifest` payload and carries a versioned manifest reference when the
prior verification signed a manifest. The reference contains only the validated manifest digest;
the object path is derived internally and is never accepted from persisted input:

`.specsync/changes/<change>/evidence/acceptance-manifests/<digest>.json`

Archived workspaces use the same path relative to their dated change directory. The object is the
deterministic JSON representation of `AcceptanceManifestV1`. Its filename digest must be lowercase
SHA-256 syntax and must equal both the validated manifest digest and the prior verification's
`acceptance_input_digest`.

## Compatibility Boundary

The public in-memory `ReopenRecord` remains hydrated with a complete `prior_verification`.
Persistence uses private versioned disk records:

- schema v1 reads the existing embedded verification unchanged;
- schema v2 reads the compact verification plus its manifest reference and hydrates the manifest
  before returning any trusted ledger;
- new writes emit schema v2 and never rewrite existing schema-v1 events merely because the ledger
  receives another approval.

Approval mutations share one location-aware loader/writer so ordinary approvals, portable
definition pairs, reopen, reaccept, migration, check, and archive cannot bypass hydration or
accidentally expand compact events.

## Immutable Object Rules

- Derive object paths from a validated digest; persisted records cannot supply filesystem paths.
- Reject traversal, symlink components, non-files, oversized payloads, unknown fields, malformed
  manifests, filename/content digest disagreement, and verification/reference disagreement.
- If an object already exists, require byte-identical deterministic content and reuse it.
- Write a new object with the repository's atomic no-follow transaction before publishing the
  referencing ledger. A failed ledger write may leave an unreferenced immutable object but cannot
  leave a trusted dangling reference.
- Never mutate or delete an existing manifest object as part of reopen, reaccept, or migration.

## Growth Bound

An A/B/A history creates at most objects A and B. Each reopening appends only bounded event and
reference metadata. Reusing A does not rewrite it, and object storage grows once per distinct
validated manifest rather than once per reopening.

## Non-goals

- No bulk conversion of historical schema-v1 reopening events.
- No compaction of the separate correction ledger.
- No change to acceptance-manifest semantics, owner resolution, freshness, or closing digests.
