---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: context
---

# Context

This is a contract normalization change discovered during final 5.0 release review. Branch-native strict checking proves frontmatter syntax, file existence, required headings, export names, and dependency target existence, but it does not prove parameter accuracy, imported-edge declaration, companion headers, or normative requirement identity.

The canonical source of truth remains under `specs/<module>/`. This workspace defines the future contract without touching those files before definition approval and implementation start. CHG-0009 remains separate release-path work; CHG-0010 must not modify its source or evidence.

Current definition state: 53 affected specs, 44 additive requirement migrations, one new configuration-header requirement, 14 dependency-frontmatter corrections, nine signature corrections, three current-configuration prose surfaces, one companion-header repair, one maturity promotion, and evidence-based task/signoff cleanup.

The first implementation dependency run exposed two analyzer false edges and one real architectural cycle. The
definition now includes code-only Rust import extraction, source-owner mapping, and independent rehash discovery so
the final graph can be complete and acyclic rather than suppressing errors.
