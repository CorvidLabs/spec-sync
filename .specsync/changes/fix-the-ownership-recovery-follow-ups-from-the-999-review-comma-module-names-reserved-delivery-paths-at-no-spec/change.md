---
id: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec
state: implementing
type: bug_fix
base_commit: 49d2326a8f80232b8fee94a5073c4c1a66fd2057
---

# Fix the ownership recovery follow-ups from the #999 review: comma module names, reserved delivery paths at no-spec approval, listing-bound silence, constraints renewal, registry specs, and directory scopes

## Intent

Fix the ownership recovery follow-ups from the #999 review: comma module names, reserved delivery paths at no-spec approval, listing-bound silence, constraints renewal, registry specs, and directory scopes

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A no-spec-change draft listing a reserved delivery path approves; a comma-bearing module name never produces a non-round-tripping printed answer; an over-bound spec listing never silently omits an owner from recovery; ownership recovery re-prompts constraints when it clears them; a registry canonical spec outside specs_dir joins owner discovery; a directory scope with a source-like extension is not ownership-checked as a file

## No-spec Rationale

Not applicable
