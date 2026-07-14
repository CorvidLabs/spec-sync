---
change: CHG-0025-address-all-unresolved-review-feedback-on-pr-366
artifact: research
---

# Research

The review threads were compared with current implementation and canonical contracts.

- `validate_effective_contracts` reconstructs `specs/<module>/<module>.spec.md` instead of reusing `canonical_module_paths`.
- `detect_source_dirs` delegates only to export-language extension detection, while coverage separately recognizes HTML, HTM, and CSS.
- `is_protected_sdd_path` omits the committed sequence ledger.
- The built-in design template emits four placeholder bullets, but strict validation recognizes only the Layout bullet.
- The verification context is checked by project checking and `change verify`, but not root `lifecycle` dispatch.
- Collision validation iterates only duplicate groups and does not restrict acknowledgements to archived records.
- `change_sequence` requires exactly four digits despite the allocator using `u64`.
- Canonical-successor evaluation recomputes the full project digest inside its candidate closure.

These findings are reproducible locally and align with the unresolved GitHub review anchors.
