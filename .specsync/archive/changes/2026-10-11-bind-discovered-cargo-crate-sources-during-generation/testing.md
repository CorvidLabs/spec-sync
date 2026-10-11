---
change: bind-discovered-cargo-crate-sources-during-generation
artifact: testing
---

# Testing

Run cargo test --test integration generate_cargo and cargo test generator::. Reproduce the fresh crate through init then generate in text, JSON, and batch modes. Inspect generated files and add export. Cover library and binary sources, workspace member isolation, named source modules, and an existing differently named spec. Keep missing configured module and retained-root replacement regressions green. Finish with fledge lanes run verify, mandatory pre-push, Attest, and fledge trust verify.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-generator-006` | `tests/integration/commands/specsync_process.rs`: four `generate_cargo` regressions pass for the fresh library, binary/combined roots, workspace isolation, existing ownership and named modules. `src/generator.rs`: retained mapping and batch/configuration boundary tests pass. |
