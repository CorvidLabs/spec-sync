---
spec: cmd_generate.spec.md
---

## Key Decisions

- **Command pattern**: load config + discover specs (`load_and_discover`), compute coverage, resolve a provider, delegate scaffolding to `generator::generate_specs_for_unspecced_modules(_paths)`, recompute coverage/validation, format output, and exit via `exit_with_status`.
- **Config triggers AI (4.4.0)**: `resolve_provider_for_generate` returns AI mode when a provider is configured *anywhere* — the `--provider` flag, `aiProvider`/`aiCommand` config, or `SPECSYNC_AI_PROVIDER`/`SPECSYNC_AI_COMMAND` env. Previously a configured provider silently fell back to templates unless `--provider` was repeated; that gate is now fixed.
- **`--provider auto` is special-cased**: `Some("auto")` is mapped to `None` so the auto-detect ladder runs even when a provider is configured; any other explicit name overrides config; an absent flag falls through to config/env/auto. Actual auto-detect (keyless local Ollama → single key → multi-key prompt/deterministic order) lives in `ai::resolve_ai_provider`.
- **Model precedence inline**: the config is cloned and `ai_model` is overridden as `--model` > `SPECSYNC_AI_MODEL` env > existing `aiModel` config before calling `ai::resolve_ai_provider`.
- **Fail fast vs. fall back**: provider *resolution* failure (unknown provider, missing key) prints the error and `process::exit(1)`; per-module AI *generation* failure falls back to the template and continues.
- **JSON mode short-circuits**: `--format json` builds the generated-paths object and exits 0 without the human-readable coverage/validation report.

## Files to Read First

- `src/commands/generate.rs` — primary source file: `cmd_generate`, `cmd_generate_all`, `cmd_generate_batch`, and the `resolve_provider_for_generate` precedence helper.
- `src/ai.rs` — `resolve_ai_provider`, `ResolvedProvider`, the auto-detect ladder, and `<PROVIDER>_API_KEY`/env precedence this command relies on.
- `src/cli.rs` (`Generate` variant) — the `provider`, `model`, `uncovered`, `batch` flag definitions and help text.
- `src/generator.rs` — `generate_specs_for_unspecced_modules(_paths)` that actually writes the spec files.

## Current Status

Fully implemented for spec-sync 4.4.0. Aligned with the reworked corvid-ai providers: AI mode is gated by configured provider (flag/config/env), `--model` is supported, and 12-factor flag > env > config precedence holds. Template-only generation remains the zero-config default.

## Notes

- This module is part of the command layer — it orchestrates library modules (`ai`, `generator`, `validator`, `output`) rather than containing domain logic.
- There are no inline `#[cfg(test)]` tests here; behavior is verified through `tests/integration.rs`.
