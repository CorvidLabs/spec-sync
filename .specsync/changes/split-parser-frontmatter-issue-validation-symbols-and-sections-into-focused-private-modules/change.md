---
id: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split parser frontmatter issue validation symbols and sections into focused private modules

## Intent

Split parser frontmatter issue validation symbols and sections into focused private modules

## Affected Canonical Specs

- `parser`

## Acceptance Criteria

- Split the parser facade, implementation and all 69 existing tests into cohesive private files each below 1000 lines; preserve original public and crate-visible API paths, whole function and method bodies, types, literals, attributes and cfg gates; retain compatibility parsing, fail-closed checked issue parsing without fallback, content-free checked errors, duplicate-key and positive-number validation, one shared delimiter scan, CRLF normalization versus borrowed stripping, byte-length-preserving fence masking, exact Unicode symbol spelling and order-preserving deduplication; retain every test and full canonical spec coverage without changing dependencies or trust/lifecycle gates

## No-spec Rationale

Not applicable
