---
change: CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors
artifact: docs
---

# Docs

- Record the parser fix under the next patch release in `CHANGELOG.md`.
- Update the parser spec and requirement to say complete first backtick symbol
  instead of backtick word.
- Note that YAML path symbols with dots and hyphens are valid documented
  exports and require no Trust-specific exception.
