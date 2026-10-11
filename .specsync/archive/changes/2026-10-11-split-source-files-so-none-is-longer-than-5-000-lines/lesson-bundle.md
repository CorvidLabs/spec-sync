# Lesson bundle — split-source-files-so-none-is-longer-than-5-000-lines

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Split source files so none is longer than 5,000 lines
- **Kind**: Refactor
- **Specs**: change, mcp, validator, manifest, github
- **Paths**: src/change.rs, src/change_tests.rs, src/github.rs, src/manifest.rs, src/mcp.rs, src/validator.rs, tests/integration/commands.rs, hi/split.md, specs/change/change.spec.md, specs/mcp/mcp.spec.md, specs/validator/validator.spec.md, specs/manifest/manifest.spec.md, specs/github/github.spec.md, src/change/sdd_version.rs, src/change/historical_definition_digest_matches.rs, src/change/collect_requirement_ids.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change_tests/substantive_plan.rs, src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs, src/change_tests/approve_rejects_a_declared_path_owned_by_an_undeclared_module.rs, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs, src/mcp/server_name.rs, src/mcp/publish_staged_file_with_verification_hooks.rs, src/validator/consumed_by_re.rs, src/validator/tests.rs, src/manifest/max_gradle_manifest_bytes.rs, src/manifest/tests.rs, tests/integration/commands/specsync_process.rs, tests/integration/commands/generate_json_honors_enforcement_strict.rs
- **Acceptance**: No source file is longer than 5,000 lines, and the program behaves the same.

## Evidence

- Verification commit: `d84243e50dc2c1d54b9e423473eb21f008915496`
- Base commit: `ec03d430b0f288d91fb7978804d1dc65abe14505`
- Verified by: `specsync check --spec change --spec github --spec manifest --spec mcp --spec validator`

## From the change's context.md

# Context

The long source files were split on an older main. This branch replays that split on current main. `include!` keeps each module one module. Cuts are on whole items, so a part is not a broken token tree. `include_str` and `#[path]` gain one `../` because rustc resolves them from the included file.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-109 | `cargo check --bin specsync --tests` compiles the included change parts. No part exceeds 5,000 lines. |
| REQ-mcp-010 | The MCP parts are the previous `src/mcp.rs` bytes, included in order. |
| REQ-validator-043 | The validator parts are the previous `src/validator.rs` bytes, included in order. |
| REQ-manifest-019 | The manifest parts are the previous body of `src/manifest.rs`, included after the module docs. |
| REQ-github-022 | The GitHub source scan includes `src/mcp/server_name.rs` and `src/mcp/publish_staged_file_with_verification_hooks.rs`. |

`cargo check --bin specsync --tests --offline` passed.

## Where these lessons go

- `specs/change/change.3md`
- `specs/mcp/mcp.3md`
- `specs/validator/validator.3md`
- `specs/manifest/manifest.3md`
- `specs/github/github.3md`
