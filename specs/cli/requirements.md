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
- As a developer, I want deterministic `specsync generate` scaffolds and separate coding-agent integrations so SpecSync needs no inference credentials
- As a developer, I want `specsync watch` to re-validate on file changes so that I get live feedback while editing
- As a developer, I want `specsync score` to grade my spec quality so that I know where to focus improvement effort
- As a team lead, I want `specsync hooks install` to set up agent instructions for Claude, Cursor, and Copilot so that AI assistants respect our specs automatically
- As a developer, I want `specsync add-spec <name>` to scaffold a single spec with companion files so that I can add documentation incrementally
- As a developer or agent, I want one `specsync change` namespace for the complete verified SDD lifecycle so that delivery state and next actions remain predictable
- As an automation author, I want the root dispatcher to honor JSON and Markdown formats for compact/archive maintenance commands instead of silently printing text

## Acceptance Criteria

- No subcommand defaults to `check`
- Exit code 0 on success, 1 on errors (or warnings in strict mode, or coverage below threshold)
- Commands that advertise JSON output suppress all ANSI color codes and emit one valid document
- Commands that advertise Markdown output produce content suitable for PR comments
- `--root <path>` allows running against a different project directory
- All domain logic is delegated to library modules — `main.rs` is purely a dispatcher and the clap grammar lives in `src/cli.rs`
- `--fix` only modifies spec files, never source code
- `init` auto-detects source directories, language, and creates a sensible default config
- `generate` accepts no provider/model options and delegates only to the deterministic generator
- A panic in any subcommand is caught and reported as a "please report it" bug message rather than a raw backtrace
- `change` dispatches every lifecycle operation through the shared domain engine and preserves structured JSON output
- `compact` and `archive-tasks` receive the resolved global `OutputFormat`; `--json` is equivalent to `--format json`

## Constraints

- Single binary with no runtime dependencies
- Must work on Linux, macOS, and Windows
- Colored output must respect `NO_COLOR` environment variable
- CLI argument parsing via clap with derive macros; grammar defined in `src/cli.rs`, dispatch in `src/main.rs`

## Out of Scope

- Interactive/TUI mode for editing specs
- GUI or web-based interface
- Daemon mode (watch is foreground only)
- Package manager plugins (npm, cargo, etc.)

### REQ-cli-001

The system SHALL expose and document the verified SDD lifecycle through the root CLI dispatcher.

Acceptance Criteria
- The CLI contract lists the `change` namespace and current initialization layout.
- Dispatch documentation includes the change lifecycle handler.

### REQ-cli-002

The CLI SHALL expose deterministic generation and agent integrations without accepting embedded model-provider or credential options.

Acceptance Criteria
- `generate` has no provider/model flags.
- MCP and agent installation commands remain available.

### REQ-cli-003

The root CLI dispatcher SHALL fail closed when a configured verification child re-enters lifecycle checking or mutation.

Acceptance Criteria

- `check`, `change`, and `lifecycle` command families consult the inherited verification context before dispatch.
- A blocked nested command exits non-zero with one actionable diagnostic.
- Commands outside the lifecycle boundary preserve current dispatch behavior.

### REQ-cli-004

The root CLI SHALL reject inherited verification re-entry before dispatching any lifecycle command handler.

Acceptance Criteria

- Explicit and default `check`, `change`, and `lifecycle` commands fail before handler-specific discovery, warnings, validation, or mutation.
- The process emits one contextual diagnostic and exits non-zero.
- Commands outside the lifecycle boundary preserve current dispatch behavior.

### REQ-cli-005

The root CLI dispatcher SHALL preserve MCP write authorization and fail closed when the configured
server root cannot be resolved.

Acceptance Criteria

- The dispatcher forwards the parsed `allow_write` capability to `run_mcp_server` without changing
  its default.
- MCP startup errors are printed to stderr and exit with usage status 2.
- No MCP request is read when server-root initialization fails.

### REQ-cli-006

The root CLI dispatcher SHALL preserve the user-requested root spelling for commands whose
retained-capability engines must detect public path replacement.

Acceptance Criteria

- MCP, check, coverage, generate, score, report, and comment receive the validated requested path
  without eager canonicalization.
- Generate retains the bound root authority through publication so a redirect after checked
  coverage returns cannot redirect an output write into the replacement.
- Other commands retain their established canonical-root dispatch behavior.
- A symlink/junction replacement after capability retention remains observable to checked
  traversal and cannot be hidden by dispatcher canonicalization.

### REQ-cli-007

The root CLI SHALL forward the resolved global output format to compact and archive-tasks handlers.

Acceptance Criteria

- `--json` and `--format json` dispatch the same `OutputFormat::Json` value.
- `--format markdown` dispatches `OutputFormat::Markdown`.
- No human banner is emitted before the structured renderer.

### REQ-cli-008

The root command dispatcher SHALL carry strict verification into the same discoverable change
workflow without adding a lifecycle profile or an external merge action.

Acceptance Criteria

- Global `--strict` adds validators to `change check` evidence and does not change lifecycle states,
  approvals, artifact layout, review, finalization, or archive behavior.
- `change status` and structured output expose the same single next action.
- `change finalize` prepares the PR for GitHub merge and never invokes a provider merge API.
- Historical repair commands remain dispatchable for existing evidence without appearing in the
  newcomer core path.

