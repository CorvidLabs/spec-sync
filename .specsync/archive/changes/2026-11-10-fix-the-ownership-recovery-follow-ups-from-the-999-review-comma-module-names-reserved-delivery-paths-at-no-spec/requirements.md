---
change: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec
artifact: requirements
---

# Requirements

- A no-spec-change draft that declares a recognized reserved delivery path (for example
  `.github/scripts/check.py` with `source_dirs = ["."]`) approves; finalize already accepts
  it under the exact delivery owner.
- A recovery command printed by the tool always runs as printed: when a module name cannot
  round-trip through the comma-separated answer, no such command is printed and the guidance
  names the modules in prose.
- When the spec listing walk exceeds its 10,000-entry bound, owner recovery says so instead
  of silently presenting an incomplete owner set.
- Ownership recovery that replaces `affected_specs` tells the operator that the constraints
  answer is cleared by that answer and must be renewed before approval.
- Owner discovery includes registry-registered canonical specs located outside `specs_dir`.
- A declared path that is a directory is not file-level ownership-checked at approve; its
  descendant files remain checked at finalize.
