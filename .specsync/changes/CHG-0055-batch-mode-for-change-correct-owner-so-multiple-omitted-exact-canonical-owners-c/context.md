---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: context
---

# Context

## Decision

Extend `change correct-owner` with a transactional batch surface while preserving the existing
append-only `AcceptanceOwnerCorrection` ledger. Each path/module pair remains an independent
sequenced audit entry; the batch only changes *when* they are persisted (validate-all, then one
atomic write). Partial validation failure must leave `state.json` untouched.

## Surfaces

1. Repeated `--path` with one `--spec` (same module for every path) or equal-length `--spec` lists.
2. `--manifest <file>`: JSON array of `{ "path", "module" }` objects, or TSV lines `path<TAB>module`.
3. `--all-missing --spec <module>`: discover production-source affected paths that currently lack any
   canonical owner and that the named module owns in frontmatter; append one correction per path.

Single-path `--path`/`--spec` remains the default one-entry form of the same API.

## Why not one audit blob

A single combined audit entry would break the contiguous sequence contract and complicate digest/
portable reconstruction. Batching is a transport/transaction concern, not a new evidence shape.

## Status

Implementing after definition approval.
