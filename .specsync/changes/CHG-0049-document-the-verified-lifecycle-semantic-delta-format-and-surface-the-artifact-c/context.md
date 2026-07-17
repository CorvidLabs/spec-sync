---
change: CHG-0049-document-the-verified-lifecycle-semantic-delta-format-and-surface-the-artifact-c
artifact: context
---

# Context

SpecSync 5.x enforces semantic-delta grammar, exact affected-module coverage, artifact completeness, requirement evidence, dependency ordering, and atomic canonical application. Those behaviors were discoverable in source, executable examples, and error messages, but they did not have a focused user-facing reference.

PR #390 added a semantic-delta reference and PR #391 added the two pre-approval gates to the quickstart. Both independently failed strict validation because their documentation paths were not owned by an active change. This change consolidates both patches on the 5.1.1 release history and gives the two documentation paths one deterministic lifecycle owner.

The scope is documentation-only. It does not change canonical specs, source behavior, public APIs, or release metadata. It depends on CHG-0048 so the documentation is verified against the stabilized 5.1.1 implementation and validation policy.
