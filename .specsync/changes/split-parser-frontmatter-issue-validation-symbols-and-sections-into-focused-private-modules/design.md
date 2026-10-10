---
change: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
artifact: design
---

# Design

src/parser.rs remains the API facade and ParsedSpec owner. Private frontmatter owns compatibility metadata parsing, scalar/list helpers, canonical borrowed stripping, and the existing shared delimiter predicate and scan. Private checked_issues owns strict serde-saphyr visitors, positive-number validation and content-free errors, importing only the scan from frontmatter. Private api_symbols owns export headers, table symbols and byte-preserving fence masking. Private sections owns heading lookup, missing and near-miss sections, and substantive-content detection. Its existing public heading helper serves api_symbols through the established parser facade.

Each owner has a separate topical test file. Sibling helpers receive only the minimum parent-scoped visibility needed. Public and crate-visible functions retain their original declarations and reexports; ParsedSpec fields do not change. No function is divided to satisfy a line limit.
