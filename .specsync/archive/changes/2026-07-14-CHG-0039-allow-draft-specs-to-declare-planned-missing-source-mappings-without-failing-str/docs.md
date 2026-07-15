---
change: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: docs
---

# Docs

Document the behavior in the Unreleased changelog: safe normalized missing paths in draft specs are planned mappings by default, remain outside current coverage, and become ordinary mappings when created or when the spec is activated. Document `require_draft_files = true` as the strict opt-in for repositories that require every draft mapping to exist immediately.

Update canonical config, commands, comment, output, types, and validator contracts so generated documentation and agent readers see the exact new field, tuple member, renderer parameter, notice channel, and validation semantics.
