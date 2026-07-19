---
change: CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c
artifact: design
---

# Design

## Domain

- Add `add_acceptance_owner_corrections(root, id, entries, actor, reason)` where each entry is
  `(path, module)`.
- Keep `add_acceptance_owner_correction` as a one-entry wrapper for compatibility.
- Shared preflight: reopen/definition checks once; validate every proposed entry against a
  provisional record that already includes prior + earlier batch entries; only then
  `write_prepared_files`.
- `--all-missing` discovery enumerates `affected_paths` that are production sources with zero
  current canonical owners and that `canonical_module_owns_exact_source_path` accepts for `--spec`.

## CLI

`ChangeAction::CorrectOwner` fields become:

- `paths: Vec<String>` (repeatable `--path`)
- `modules: Vec<String>` (repeatable `--spec`)
- `manifest: Option<PathBuf>`
- `all_missing: bool`
- `actor`, `reason` unchanged

Clap rejects empty selection (no paths, no manifest, no `--all-missing`) and conflicting modes
before domain mutation.

## Adapter

`cmd_change` resolves the entry list, calls the batch domain API, and prints how many corrections
were appended (JSON still emits the full persisted record).
