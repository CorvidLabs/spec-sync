---
change: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: docs
---

# Docs

Document the behavior in the Unreleased changelog: safe missing paths in draft specs are planned mappings by default, remain outside current coverage, and become ordinary mappings when created or when the spec is activated. Document `require_draft_files = true` as the strict opt-in for repositories that require every draft mapping to exist immediately.
