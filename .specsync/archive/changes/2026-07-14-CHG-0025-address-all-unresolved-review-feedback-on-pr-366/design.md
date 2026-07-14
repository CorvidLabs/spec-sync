---
change: CHG-0025-address-all-unresolved-review-feedback-on-pr-366
artifact: design
---

# Design

## Canonical paths and successor evaluation

Effective-contract validation will call the same registry-aware canonical module resolver used by semantic-delta preparation. Registered paths remain confined to the project root and conventional module paths remain the fallback. Canonical-successor evaluation will calculate the current project digest once before iterating candidates and reuse it only for verifying candidates.

## Sequence integrity

Change IDs accept a minimum four-digit numeric sequence with no artificial upper width. The committed `.specsync/change-sequence.json` ledger becomes a protected SDD path that cannot be hidden by `.specsync/` ignore policy. An acknowledged collision is valid only when its exact ID set consists entirely of immutable accepted or archived records. Missing IDs, added IDs, surviving singletons, and any draft, approved, implementing, or verifying member fail closed.

## Recursive command boundary

The inherited verification-context marker is checked at root CLI dispatch for `check`, `change`, and `lifecycle` command families. A wrapper that indirectly invokes any of these lifecycle surfaces therefore exits once before nested enforcement or mutation can run.

## Static discovery and scaffold completeness

Zero-config source-directory detection uses the same default measurable extension set as coverage, including HTML, HTM, and CSS. Companion validation enumerates every line emitted by the built-in design template for Layout, Components, Tokens, and Assets while continuing to ignore fenced examples and similar prose.
