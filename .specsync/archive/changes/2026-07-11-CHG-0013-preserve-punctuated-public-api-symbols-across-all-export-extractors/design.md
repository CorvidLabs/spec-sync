---
change: CHG-0013-preserve-punctuated-public-api-symbols-across-all-export-extractors
artifact: design
---

# Design

Parse the complete first nonempty backtick-delimited cell value on a Markdown
table row. Do not maintain a second character allowlist in the spec parser: the
extractors are the authority on valid symbol spelling, and a delimiter-based
parser naturally preserves punctuation used by current and future languages.

Retain the table-row anchor, first-symbol rule, Public API boundary,
export-subsection allowlist, method/property exclusions, deduplication, and
order preservation. Reject empty or whitespace-only delimiters and rows without
a closing backtick.
