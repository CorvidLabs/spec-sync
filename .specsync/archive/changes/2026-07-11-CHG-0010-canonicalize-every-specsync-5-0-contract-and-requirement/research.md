---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: research
---

# Research

The branch-native audit covered 62 canonical specs and 312 spec/companion files. Strict checking passed 62/62 specs with 105/105 source files and 68,700/68,700 LOC covered, while score reported a 99.9 average and 62 A grades. Those structural gates do not validate parameter lists, companion frontmatter, normative requirement identity, or import completeness.

The deeper audit found 35 undeclared import edges across 14 modules, nine stale Public API parameter cells, 44 requirements companions without stable IDs or SHALL language, one missing companion frontmatter block, and one shipped module (`cmd_migrate`) whose draft status causes strict section/export validation to be skipped. The exact union of those inventories is 53 affected specs; nine already-correct modules are intentionally excluded.

Semantic deltas can add requirements and replace Markdown sections, but `depends_on` is YAML frontmatter. Dependency frontmatter therefore remains an explicit, reviewed implementation edit recorded in each affected delta's truthful Dependencies section. Acceptance must run `specsync deps --strict` in addition to ordinary strict checking.
