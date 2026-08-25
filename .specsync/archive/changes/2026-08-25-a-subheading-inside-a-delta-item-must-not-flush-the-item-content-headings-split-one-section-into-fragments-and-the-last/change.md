---
id: a-subheading-inside-a-delta-item-must-not-flush-the-item-content-headings-split-one-section-into-fragments-and-the-last
state: archived
type: bug_fix
base_commit: 875752ee991d458db172dec6ceb712462fe2a614
---

# A subheading inside a delta item must not flush the item: content headings split one section into fragments and the last one wins

## Intent

A subheading inside a delta item must not flush the item: content headings split one section into fragments and the last one wins

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a delta section carrying content subheadings parses as ONE item holding all of them, not one item per subheading
- a real item heading still ends the previous item, so distinct sections are not merged
- a delta declaring the same operation, target and key twice is refused rather than silently keeping the last
- the living spec no longer loses documented behaviour a change never touched

## No-spec Rationale

Not applicable
