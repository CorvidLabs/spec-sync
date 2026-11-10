---
id: bounded-failure-output-never-splits-a-multi-byte-character-and-gate-refusal-evidence-is-tested
state: archived
type: bug_fix
base_commit: aa2f5108929c19a4e9eabd126985de609f709af1
---

# Bounded failure output never splits a multi-byte character and gate-refusal evidence is tested

## Intent

bounded failure output never splits a multi-byte character and gate-refusal evidence is tested

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- change check records findings over 4 KiB that contain multi-byte characters at the cut without panicking, keeping a tail that starts on a character and line boundary after a leading … line; the gate-refusal (effective contract) evidence entry's output carries the refusal texts and a test asserts it

## No-spec Rationale

Not applicable
