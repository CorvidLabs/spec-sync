---
change: refuse-incomplete-configured-root-coverage-after-cargo-discovery-loss-for-specsync-6-1-1
artifact: design
---

# Design

Retain the existing checked-Cargo-only admission of exact regular root source files. A configured eligible regular file cannot silently become a skipped directory when manifest discovery loses its exact-target authority: return an ordinary inconclusive coverage error instead. This also covers absent manifests, which can legitimately produce an empty discovery result without a parse error. Do not turn an explicit file into a recursive project-root scan or admit arbitrary configured files.

Keep directory-only configured fallback, filtering and existing capability/identity checks. Excluded or non-source candidates retain existing filtering order. Do not open a symlink, outside root, nonregular object or unknown source merely to recover a denominator. Valid checked Cargo targets and ordinary recursive directories retain their stated selections and current budgets.

The user-visible failure uses existing text/JSON error conventions. No new CLI option, dependency, persisted format, owner mapping or lifecycle bypass is introduced. Independent review compares the final source/test hashes and actual before/after receipts.
