---
spec: cmd_new.spec.md
---

## Key Decisions

- **Scaffold, don't author**: `cmd_new` writes frontmatter and section skeletons but leaves prose, invariants, and dependency descriptions to the author. Public API rows are review prompts, not finished docs.
- **Shared renderer**: source discovery, required sections, valid empty-file frontmatter, and Public API population are delegated to `generator`.
- **Never clobber**: an existing target spec aborts with exit 1 — creation is non-destructive.

## Files to Read First

- `src/commands/new.rs` — command orchestration and non-overwrite behavior.
- `src/generator.rs` — shared source discovery, spec rendering, export population, and companion generation.

## Current Status

Stable and implemented. Integration tests cover basic creation, all required sections, source auto-detection, no-match guidance, and
module-name safety. The command module has no inline `#[cfg(test)]` module; explicit `--full` and refuse-overwrite
integration fixtures remain open.

## Notes

- This is a command-layer module: it orchestrates `config` and `generator` rather than holding domain logic.
- `depends_on` is always emitted empty; imports are not analyzed to infer dependencies.
