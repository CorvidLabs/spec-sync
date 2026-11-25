# Lesson bundle: a-requirement-whose-shall-sits-at-a-line-edge-counts-as-a-normative-statement

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A requirement whose SHALL sits at a line edge counts as a normative statement
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests/approved_delta_requirements.rs
- **Acceptance**: a requirement whose SHALL ends or starts a wrapped line is accepted; a requirement with no SHALL is still refused with the normative SHALL message

## Evidence

- Verification commit: `f690327c2f979c708647a5890925dfe5dfeaed7d`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

#1076, found while recording #434's REQ-change-164: `validate_requirement`
(`src/change/collect_requirement_ids.rs`) accepted a requirement only when it contained `" SHALL "` or started
with `"SHALL "`. A requirement wrapped at a normal width, with SHALL at a line edge, was refused at
`change approve` as having no normative SHALL. Now SHALL counts as a whole word across any whitespace.
Lane A (t38c) agreed this lane takes it.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through a
Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).

## From the change's testing.md

# Testing

| Behaviour | Evidence |
|---|---|
| SHALL at a line edge is normative | `change::tests::a_shall_at_a_line_edge_is_a_normative_statement` (red, then green) |
| No SHALL is still refused | the same test |

Locally: `cargo test --bin specsync approved_delta`; hosted CI runs the full suite at the archived head.

## Where these lessons go

- `specs/change/change.3md`
