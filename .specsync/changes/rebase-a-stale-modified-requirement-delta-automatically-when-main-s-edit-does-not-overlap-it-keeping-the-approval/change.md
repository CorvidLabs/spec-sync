---
id: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
state: implementing
type: feature
base_commit: 99f61b2b5667d60c3b0705902486f910c8e067c5
---

# Rebase a stale MODIFIED requirement delta automatically when main's edit does not overlap it, keeping the approval

## Intent

Rebase a stale MODIFIED requirement delta automatically when main's edit does not overlap it, keeping the approval

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- When main edits a requirement a MODIFIED delta was approved against and the edits do not overlap word by word, change check writes the word merge into the delta as a proposal and refuses until a person approves it with change approve, after which change check applies it; when the edits overlap, main deleted the requirement or edited its heading, the working tree holds other text, the requirement is too long, or Git cannot read main's text, it refuses as today naming the cause and the way on, and writes nothing; nothing is proposed in CI

## No-spec Rationale

Not applicable
