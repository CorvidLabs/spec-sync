---
spec: cli.spec.md
---

## User Stories

- As a developer, I want to run `specsync` with no arguments and get validation results so that the default behavior is useful without memorizing commands
- As a CI engineer, I want `--strict` mode to fail the build on warnings so that spec quality is enforced in pipelines
- As a CI engineer, I want `--require-coverage N` to fail if file coverage drops below a threshold so that we maintain documentation standards
- As an AI agent, I want `--json` output so that I can parse validation results programmatically
- As a developer, I want `--fix` to auto-add undocumented exports to my specs so that keeping specs current is low-friction
- As a developer, I want `specsync init` to scaffold a config with auto-detected settings so that getting started takes seconds
- As a developer, I want `specsync generate` to create specs for all unspecced modules in one command so that I can bootstrap documentation for an existing project
- As a developer, I want `specsync watch` to re-validate on file changes so that I get live feedback while editing
- As a developer, I want `specsync score` to grade my spec quality so that I know where to focus improvement effort
- As a team lead, I want `specsync hooks install` to set up agent instructions for Claude, Cursor, and Copilot so that AI assistants respect our specs automatically
- As a developer, I want `specsync add-spec <name>` to scaffold a single spec with companion files so that I can add documentation incrementally

## Acceptance Criteria

- [ ] No subcommand defaults to `check`
- [ ] Exit code 0 on success, 1 on errors (or warnings in strict mode, or coverage below threshold)
- [ ] `--json` suppresses all ANSI color codes and outputs valid JSON
- [ ] `--format markdown` produces output suitable for PR comments
- [ ] `--root <path>` allows running against a different project directory
- [ ] All domain logic is delegated to library modules — main.rs is purely a dispatcher
- [ ] `--fix` only modifies spec files, never source code
- [ ] `init` auto-detects source directories, language, and creates a sensible default config

## Constraints

- Single binary with no runtime dependencies
- Must work on Linux, macOS, and Windows
- Colored output must respect `NO_COLOR` environment variable
- CLI argument parsing via clap with derive macros

## Out of Scope

- Interactive/TUI mode for editing specs
- GUI or web-based interface
- Daemon mode (watch is foreground only)
- Package manager plugins (npm, cargo, etc.)
