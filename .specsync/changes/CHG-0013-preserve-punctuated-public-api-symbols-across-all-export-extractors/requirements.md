---
change: CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors
artifact: requirements
---

# Requirements

### REQ-change-parser-001

The validator SHALL match a documented Public API symbol exactly as emitted by
the selected language extractor when the complete symbol appears in the first
backtick-delimited cell of a recognized table row.

Acceptance Criteria
- Dotted and hyphenated GitHub Actions YAML paths match exactly.
- Ordinary identifiers remain unchanged.
- Only the first nonempty backtick symbol in a table row is collected.
- Informational and member subsections retain their existing exclusions.
- Malformed rows and empty delimiters do not produce symbols.
- A temporary active Trust spec documents all 30 extracted exports under strict validation.
