---
spec: cmd_generate.spec.md
---

## User Stories

- As a developer, I want `specsync generate` to scaffold spec files for unspecced modules so that I can reach coverage quickly
- As a developer with an AI provider configured, I want `generate` to use AI automatically — without re-passing `--provider` — so that a configured provider "just works"
- As a developer, I want `--provider auto` to force fresh auto-detection even when a provider is configured so that I can override config on the fly
- As a developer, I want `--model` to override the configured/env model for a single run so that I can experiment without editing config
- As a developer, I want a template-only fallback when no provider is configured anywhere so that `generate` still works with zero AI setup
- As a developer, I want `--batch mod1,mod2` to generate only specific modules so that I can target a subset
- As a CI operator, I want `--format json` machine-readable output and meaningful exit codes so that pipeline steps are actionable

## Acceptance Criteria

- AI mode is entered when a provider is configured anywhere: the `--provider` flag, `aiProvider`/`aiCommand` in config, or the `SPECSYNC_AI_PROVIDER`/`SPECSYNC_AI_COMMAND` env vars (`resolve_provider_for_generate` gates on exactly these). Repeating `--provider` is no longer required.
- With nothing configured anywhere → template-only generation (no AI call).
- `--provider auto` ignores the configured provider and forces the auto-detect ladder; an explicit `--provider <name>` overrides config; an absent flag falls through to config/env/auto.
- Model precedence is `--model` > `SPECSYNC_AI_MODEL` env > `aiModel` config > corvid-ai default, applied by cloning the config and overriding `ai_model` before resolution.
- The header prints "Generating Specs (AI)" when a provider resolved, else "Generating Specs".
- `--batch` expands comma-separated and space-separated entries; modules already specced are skipped, modules absent from the coverage report are reported as not-found.
- `--uncovered` is accepted but behaves identically to the default (generate for all unspecced modules).
- `--format json` prints a `{ "generated": [...] }` object (batch mode also includes `requested`, `skipped_already_specced`, `skipped_not_found`) and exits 0 without the human-readable report.
- After generation, coverage and validation are recomputed so the summary reflects the new specs.
- If AI provider resolution fails (unknown provider, missing API key), the process prints the error and exits 1.
- If AI generation fails for an individual module, generation falls back to template and continues (does not abort the run).

## Constraints

- Must not panic on expected error conditions — print the error and exit, or fall back to templates
- Must work with the project's Clap-based CLI argument parsing (`Generate { provider, model, uncovered, batch }`)
- Provider/model resolution must follow 12-factor precedence (flag > env > config), delegating to `ai::resolve_ai_provider`

## Out of Scope

- GUI or web interface
- Interactive prompts here (the multi-key auto-detect TTY prompt lives in the `ai` module, not `cmd_generate`)
- Defining the AI providers themselves (owned by the `ai`/corvid-ai layer)
- Editing or refining specs after scaffolding (that is `score`/`refine`/validation territory)
