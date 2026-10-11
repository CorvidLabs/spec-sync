# Lesson bundle — bind-discovered-cargo-crate-sources-during-generation

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Bind discovered Cargo crate sources during generation
- **Kind**: BugFix
- **Specs**: generator, validator, types, cli, output, comment
- **Paths**: src/generator.rs, src/validator/tests.rs, src/types.rs, src/main.rs, src/output.rs, src/comment.rs, tests/integration/commands/specsync_process.rs
- **Acceptance**: The issue #813 minimal Cargo library succeeds through init then generate, creates specs/demo/demo.spec.md bound to src/lib.rs and documents add. Text, JSON, and batch generation bind discovered Cargo sources; binary and workspace crates are scoped correctly; existing spec-owned files remain owned; missing configured modules still fail with skipped_no_files.

## Evidence

- Verification commit: `beca555b8421b171b1c630d03efb8fc10b208a20`
- Base commit: `9003fad7ec4dea3938179d076248b48f29b6ed48`
- Verified by: `specsync check --spec cli --spec comment --spec generator --spec output --spec types --spec validator`

## From the change's context.md

# Context

Issue #813 reproduces on released 6.1.0: init finds src, coverage names the Cargo package demo and src/lib.rs, but generate matches only directory/file names and skips demo. The user explicitly requested fixing the issue. Coverage already has checked manifest mappings; preserve them rather than reopening Cargo.toml during generation.

## From the change's design.md

# Design

CoverageReport carries manifest module source paths observed through the retained coverage capability. Generation resolves configured mappings and conventional name-based paths first, then the recorded manifest paths, intersecting every binding with measured uncovered files. More specific named modules keep their files when a manifest crate spans their directories. All generation entry points use the shared coverage binding; existing destinations remain untouched.

## From the change's testing.md

# Testing

Run cargo test --test integration generate_cargo and cargo test generator::. Reproduce the fresh crate through init then generate in text, JSON, and batch modes. Inspect generated files and add export. Cover library and binary sources, workspace member isolation, named source modules, and an existing differently named spec. Keep missing configured module and retained-root replacement regressions green. Finish with fledge lanes run verify, mandatory pre-push, Attest, and fledge trust verify.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-generator-006` | `tests/integration/commands/specsync_process.rs`: four `generate_cargo` regressions pass for the fresh library, binary/combined roots, workspace isolation, existing ownership and named modules. `src/generator.rs`: retained mapping and batch/configuration boundary tests pass. |

## Where these lessons go

- `specs/generator/generator.3md`
- `specs/validator/validator.3md`
- `specs/types/types.3md`
- `specs/cli/cli.3md`
- `specs/output/output.3md`
- `specs/comment/comment.3md`
