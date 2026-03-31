# SpecSync v3 Scope

This document defines what is in scope for v3 and what is explicitly deferred.

## In Scope (Shipped)

### Core Validation
- Bidirectional spec-to-code validation (check, coverage, score)
- Multi-language export extraction (TS, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby)
- Required section enforcement, frontmatter validation
- Quality scoring with letter grades and improvement suggestions
- `--strict` and `--require-coverage` for CI gating

### AI-Powered Generation
- `specsync generate --provider <name>` for AI-powered spec generation
- `--provider auto` for auto-detection of installed providers
- Provider support: Claude CLI, Ollama, Copilot, Anthropic API, OpenAI API
- Config-based provider resolution (aiProvider, aiCommand, aiModel, aiTimeout)
- Template fallback when AI is unavailable or fails

### Cross-Project References
- `depends_on: ["owner/repo@module"]` syntax in spec frontmatter
- `specsync resolve` — local dependency resolution with existence checks
- `specsync resolve --remote` — opt-in remote registry fetching via GitHub
- `specsync-registry.toml` — declares available specs per repo
- `specsync init-registry` — auto-generates registry from existing specs
- Cross-project refs are **metadata only** in `specsync check` (no CI cost)

### Companion Files
- `requirements.md` — product requirements, user stories, acceptance criteria (authored by Product/Design)
- `tasks.md` — checkbox-driven work tracking, multi-role sign-offs (Product, QA, Design, Dev)
- `context.md` — agent briefing with key decisions, files to read, status
- Auto-generated alongside every new spec via `generate` and `add-spec`

### CLI
- `check` — validate specs (default command)
- `coverage` — file and LOC coverage report
- `generate` — scaffold specs (template or AI)
- `score` — quality scoring
- `resolve` — dependency resolution (local + optional remote)
- `add-spec` — scaffold a single new spec with companions
- `init` — create config file
- `init-registry` — create registry file
- `watch` — continuous validation on file changes
- `mcp` — MCP server mode for AI agent integration
- All commands support `--json` output

### VS Code Extension
- Real-time spec validation with inline diagnostics (errors + warnings)
- CodeLens quality scores on spec files
- Coverage and scoring webview reports with VS Code theme integration
- Five commands: Validate Specs, Show Coverage, Score Quality, Generate Specs, Initialize Config
- Persistent status bar with pass/fail/error state indicators
- Debounced validate-on-save (500ms)
- Configurable binary path, validate-on-save toggle, inline score toggle
- Published on VS Code Marketplace as `corvidlabs.specsync`

### Configuration
- `specsync.json` (JSON) and `.specsync.toml` (TOML) config formats

## Out of Scope (Deferred / Not Planned)

### Dependency Graph Visualization
Not building a visual dep graph. `specsync resolve` gives a text listing.
If users want a graph, they can pipe `--json` output to a graphing tool.

### Automatic Cross-Repo CI Validation
Every repo checking every other repo's references in CI is explicitly **not** in scope.
Cross-project refs are declarative metadata. `--remote` is opt-in and meant for
manual or periodic checks, not default CI runs.

### Registry Federation / Discovery
No central registry service. Each repo hosts its own `specsync-registry.toml`.
Discovery is manual (you know which repos you depend on).

### Spec Diffing / Migration
No automatic spec migration between versions. Specs are human-authored documents.

### Lock Files / Version Pinning
No lock file for cross-project dependencies. Refs point to HEAD.
Pinning to specific versions is not planned.

## Design Principles

1. **Zero CI cost by default** — `specsync check` never hits the network
2. **Opt-in complexity** — remote resolution, AI generation, etc. are all flags
3. **Language-agnostic** — works with any codebase that has source files
4. **Human-first, AI-friendly** — specs are readable markdown, parseable by agents
5. **Minimal config** — works out of the box with sensible defaults
