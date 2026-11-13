---
id: the-persistent-read-cache-stays-bounded
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The persistent read cache stays bounded

## Intent

The persistent read cache stays bounded

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- after an archive-introduction index is built the cache file holds no approvals.json bytes, and a warm index reads back the same ledgers as a live one; a flush keeps at most 4096 archive-introduction entries and keeps those the command used; a cache file over 8 MiB is not loaded; verdicts and warm-read process bounds are unchanged

## No-spec Rationale

Not applicable
