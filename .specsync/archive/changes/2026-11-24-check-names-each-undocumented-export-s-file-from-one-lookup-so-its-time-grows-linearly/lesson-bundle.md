# Lesson bundle: check-names-each-undocumented-export-s-file-from-one-lookup-so-its-time-grows-linearly

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Check names each undocumented export's file from one lookup, so its time grows linearly
- **Kind**: BugFix
- **Specs**: validator
- **Paths**: src/validator/consumed_by_re.rs, tests/integration/check_scaling.rs, tests/integration.rs
- **Acceptance**: with four times as many undocumented exports, check takes well under seven times as long, and each warning names the same file as before

## Evidence

- Verification commit: `98fb4369f038d229a81c88c6f4f1a63430ca2c96`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec validator`

## From the change's context.md

# Context

#574: `check` time grew with the square of the number of undocumented exports (the issue measured
160 s for 400k; on 2026-11-11 a debug build doubled the count and took 4.2 times as long). The warning
loop in `src/validator/consumed_by_re.rs` named each undocumented export's file with
`exports_by_file.iter().find(...)`, a scan of every export per warning. Now the first file for each
symbol goes into a map once. The output is identical: the same warnings name the same first file.
Leif approved it on 2026-11-13 ("#574 fast check", coordinator chat 14:03Z, relayed by
claude-code-t370).

The only observable difference is time, so the red test compares 8,000 and 32,000 undocumented
exports on the same machine. On main the ratio was 9.0 (547 ms vs 4.91 s); the test allows under 7.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).

## From the change's testing.md

# Testing

| Behaviour | Evidence |
|---|---|
| check time grows linearly in undocumented exports (SPEED-1.b) | `check_scaling::check_time_grows_linearly_with_undocumented_exports` (red: ratio 9.0; green after) |
| warnings unchanged | the existing fix:: and check:: integration tests |

The timing test runs under the shared build lock locally; hosted CI runs it in the full suite.

## Where these lessons go

- `specs/validator/validator.3md`
