---
change: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
artifact: research
---

# Research

`src/schema.rs` already lists `mjs` and `cjs` among JavaScript schema extensions. In contrast, `Language::from_extension` and `Language::TypeScript.extensions()` stop at `ts`, `tsx`, `js`, `jsx`, `mts`, and `cts`. The validator's default `has_extension` path relies on the shared `Language` registry, explaining why mapped module files can be present in spec frontmatter but absent from measured totals. Aligning the registry is smaller and safer than adding validator-only exceptions because generation and all other default-discovery consumers inherit the same correction.
