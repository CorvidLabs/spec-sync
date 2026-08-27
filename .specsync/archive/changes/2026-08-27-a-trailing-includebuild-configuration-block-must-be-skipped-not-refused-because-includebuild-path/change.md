---
id: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
state: archived
type: bug_fix
base_commit: fe55a2179ca298ad1ca4e8fc7b7465890b85cc75
---

# A trailing includeBuild configuration block must be skipped, not refused, because includeBuild(path) { dependencySubstitution { ... } } is the common spelling and contributes no module

## Intent

A trailing includeBuild configuration block must be skipped, not refused, because includeBuild(path) { dependencySubstitution { ... } } is the common spelling and contributes no module

## Affected Canonical Specs

- `manifest`

## Acceptance Criteria

- parse_gradle_settings accepts includeBuild("vendor/shared") { dependencySubstitution { ... } } — one line and multi-line — and the composite build contributes no module, so a sibling include(":app") is still discovered. A real Gradle fixture using that spelling runs coverage successfully and reports its modules instead of degrading to a manifest notice or an inconclusive gate. Every refusal that guarded the path survives with an argument-naming diagnostic: includeBuild("../outside") { ... } and every rooted or traversing path still fails beneath-the-root, interpolated and dynamic arguments still fail dynamic, more than one path argument still fails one-literal-path, and an unbalanced trailing block fails with an unbalanced-braces diagnostic. Braces inside string literals and inside comments do not move the balance scan in either direction. Everything inside the block remains subject to every other guard: a block-scoped include, a projectDir mutation, or an unrecognized project mutation written inside the block still fails closed.

## No-spec Rationale

Not applicable
