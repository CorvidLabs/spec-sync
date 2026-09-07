---
id: make-ship-status-product-stage-completion-use-current-verification-content-consistently
state: implementing
type: bug_fix
base_commit: 249a3e0db7e083569440e864501c416072feb5a6
---

# Make ship-status product-stage completion use current verification content consistently

## Intent

Make ship-status product-stage completion use current verification content consistently

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- Product-stage completion uses the same verification-content currency predicate as readiness; unchanged verified content after rewritten ancestry reports product verification done; altered content still reports incomplete; text and JSON agree; review currency and finalization policy remain enforced.

## No-spec Rationale

Not applicable
