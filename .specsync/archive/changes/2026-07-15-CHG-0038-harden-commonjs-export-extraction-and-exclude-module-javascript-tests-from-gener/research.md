---
change: CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener
artifact: research
---

# Research

The current property regex consumes one right-hand-side byte, which hides a second
assignment in a chain. Literal masking handles strings, templates, and comments but
not JavaScript regex literals. Tree-sitter is already available and can classify a
matched byte range's ancestors without replacing the regex-mode extractor. Both
generated-spec walkers already have the project root required by `is_test_file`.
