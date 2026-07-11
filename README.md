<div align="center">

# SpecSync

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-SpecSync-blue?logo=github)](https://github.com/marketplace/actions/spec-sync)
[![spec coverage](https://img.shields.io/endpoint?url=https://corvidlabs.github.io/spec-sync/badges/coverage.json)](https://corvidlabs.github.io/spec-sync/)
[![CI](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml/badge.svg)](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/specsync.svg)](https://crates.io/crates/specsync)
[![Downloads](https://img.shields.io/crates/d/specsync.svg)](https://crates.io/crates/specsync)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Turn requirements into module contracts that fail CI when code drifts—without losing the decisions and evidence around the fix.**

Rust · single binary · 33 languages · no SpecSync API key required

[Quick start](#quick-start) · [Full SDD lifecycle](#full-sdd-lifecycle) · [Documentation](https://corvidlabs.github.io/spec-sync/docs/) · [Examples](https://corvidlabs.github.io/spec-sync/examples/) · [Comparisons](#how-it-compares)

</div>

---

## A contract change in 60 seconds

Start with product intent in `specs/auth/requirements.md`:

```markdown
### REQ-auth-004
The system SHALL let a signed-in user revoke every active session.
```

Refine it into the durable module contract in `specs/auth/auth.spec.md`:

```markdown
| Name | Kind | Description |
|---|---|---|
| `revoke_all_sessions` | function | Revokes every session owned by the current user. |
```

Now a developer adds a second public export without updating the contract:

```rust
pub fn revoke_all_sessions(user_id: UserId) -> Result<usize, SessionError> { /* ... */ }
pub fn revoke_session(session_id: SessionId) -> Result<(), SessionError> { /* new */ }
```

The same check runs locally and in CI:

```console
$ specsync check --strict
specs/auth/auth.spec.md
  ⚠ undocumented export `revoke_session`

1 warning treated as an error in strict mode
```

The fix preserves more than a generated document:

```text
requirements.md  why the behavior exists and how success is judged
auth.spec.md      the module/API contract checked against code
context.md        decisions, constraints, and files the next agent needs
testing.md        requirement-to-test evidence
CHG-*/            approved deltas, verification, and the delivery audit trail
```

Add the missing contract row—or make the export private—then rerun the check. CI turns green while the requirement, context, evidence, and exact contract change remain reviewable in Git.

[Read the adversarial proof](https://corvidlabs.github.io/spec-sync/docs/comparisons/adversarial-proof/)

## What SpecSync catches

SpecSync validates Markdown module specs (`*.spec.md`) against source code in both directions.

| Drift | Result |
|---|---|
| Code exports something absent from its spec | Warning; fails in strict mode |
| A spec documents an export missing from code | Error |
| A referenced source file was deleted | Error |
| A required spec section is missing | Error |
| A declared dependency does not exist | Error |
| A source import is undeclared | Strict dependency error |
| A documented database table or column is missing | Error |
| A schema column is undocumented or has a type mismatch | Warning |

It also provides a verified spec-driven development lifecycle, coverage gates, quality scoring, dependency analysis, cross-project references, Git hooks, editor integration, and agent-native workflows.

SpecSync's core is deterministic and local. It does not require a hosted SpecSync service, Corvid AI account, provider key, or embedded model. Claude, Cursor, Codex, Gemini, and other coding agents use the same CLI and lifecycle through their own permissions.

## Full SDD lifecycle

SpecSync 5.0 manages delivery as versioned change workspaces:

```text
draft → approved → implementing → verifying → accepted → archived
```

```bash
specsync change new "Add passkeys" --spec auth --path src/auth.rs --json
specsync change answer CHG-0001-add-passkeys acceptance_criteria \
  "A registered passkey authenticates the user" --json
specsync change approve CHG-0001-add-passkeys
specsync change start CHG-0001-add-passkeys

# implement the approved contract or semantic delta
specsync check --strict
specsync change verify CHG-0001-add-passkeys
specsync change accept CHG-0001-add-passkeys

# merge first; archive after delivery integration is proven
specsync change archive CHG-0001-add-passkeys
```

Approvals are human, portable, and digest-bound. Verification binds test evidence to the exact commit and working-tree inputs. Acceptance atomically updates canonical specs and requirements. Dirty edits invalidate evidence instead of silently changing the accepted result.

[Read the workflow guide](https://corvidlabs.github.io/spec-sync/docs/workflow/) or run the [complete lifecycle example](examples/sdd-lifecycle/).

## Install

### Cargo

```bash
cargo install specsync
```

### GitHub Action

```yaml
- uses: CorvidLabs/spec-sync@v5
  with:
    strict: 'true'
    require-coverage: '100'
```

`@v5` follows compatible 5.x Action updates. Pin both the Action and binary for an immutable install:

```yaml
- uses: CorvidLabs/spec-sync@v5.0.0
  with:
    version: '5.0.0'
```

### Pre-built binaries

Download macOS, Linux, or Windows binaries from [GitHub Releases](https://github.com/CorvidLabs/spec-sync/releases).

## Quick start

```bash
# Initialize configuration and the verified lifecycle
specsync init

# Scaffold a module contract and companion files
specsync add-spec auth

# Validate contract ↔ code in both directions
specsync check --strict

# Measure coverage and spec quality
specsync coverage
specsync score --all

# Install native coding-agent workflows and Git hooks
specsync agents install
specsync hooks install
```

For an existing 4.x project, use the guided migration and adoption flow described in the [configuration](https://corvidlabs.github.io/spec-sync/docs/configuration/) and [workflow](https://corvidlabs.github.io/spec-sync/docs/workflow/) guides.

## Specs and companion files

Each module keeps one executable contract and focused context beside it:

```text
specs/auth/
├── auth.spec.md      validated module contract
├── requirements.md  stable requirements and acceptance criteria
├── tasks.md         active work, roadmap, and test debt
├── context.md       architectural decisions and current state
└── testing.md       automated, manual, and edge-case evidence
```

| Artifact | Durable responsibility |
|---|---|
| `*.spec.md` | Source files, public API, invariants, behavior, errors, dependencies, and change history |
| `requirements.md` | Stable `REQ-*` identities, normative SHALL statements, and acceptance criteria |
| `tasks.md` | Work still to do; requirements are not checkboxes |
| `context.md` | Decisions, constraints, key files, and handoff state |
| `testing.md` | Requirement traceability, automated coverage, manual QA, and adversarial cases |
| `.specsync/changes/CHG-*` | Proposed deltas, approvals, verification, and closing evidence |

[Read the complete spec format](https://corvidlabs.github.io/spec-sync/docs/spec-format/), [companion-file reference](https://corvidlabs.github.io/spec-sync/docs/companion-files/), and [workflow conventions](https://corvidlabs.github.io/spec-sync/docs/workflow/).

## Documentation

| Start here | Reference and integration |
|---|---|
| [Why SpecSync?](https://corvidlabs.github.io/spec-sync/docs/why-specsync/) | [CLI reference](https://corvidlabs.github.io/spec-sync/docs/cli/) |
| [Quick start](https://corvidlabs.github.io/spec-sync/docs/quickstart/) | [Configuration](https://corvidlabs.github.io/spec-sync/docs/configuration/) |
| [Workflow guide](https://corvidlabs.github.io/spec-sync/docs/workflow/) | [Spec format](https://corvidlabs.github.io/spec-sync/docs/spec-format/) |
| [Companion files](https://corvidlabs.github.io/spec-sync/docs/companion-files/) | [Language reference](https://corvidlabs.github.io/spec-sync/languages/) |
| [Architecture](https://corvidlabs.github.io/spec-sync/docs/architecture/) | [Cross-project references](https://corvidlabs.github.io/spec-sync/docs/cross-project-refs/) |
| [AI and coding agents](https://corvidlabs.github.io/spec-sync/docs/integrations/ai-agents/) | [GitHub Action](https://corvidlabs.github.io/spec-sync/docs/integrations/github-action/) |
| [Examples](https://corvidlabs.github.io/spec-sync/examples/) | [VS Code extension](https://corvidlabs.github.io/spec-sync/docs/integrations/vscode-extension/) |

## Executable examples

The examples create disposable projects and run the real CLI:

- [Complete SDD lifecycle](examples/sdd-lifecycle/)
- [Five evolving product epics](examples/sdd-five-epics/)
- [Ordered concurrent changes](examples/sdd-concurrent-changes/)
- [CI gate](https://corvidlabs.github.io/spec-sync/examples/ci-gate/)
- [Polyglot project](https://corvidlabs.github.io/spec-sync/examples/polyglot/)
- [Rust workspace](https://corvidlabs.github.io/spec-sync/examples/rust-workspace/)

## How it compares

SpecSync can stand alone or enforce the implementation layer beneath planning-oriented tools:

- [SpecSync vs. Spec Kit](https://corvidlabs.github.io/spec-sync/docs/comparisons/spec-kit/)
- [SpecSync vs. OpenSpec](https://corvidlabs.github.io/spec-sync/docs/comparisons/openspec/)
- [Use SpecSync, Spec Kit, and OpenSpec together](https://corvidlabs.github.io/spec-sync/docs/comparisons/using-together/)
- [Adversarial detection and knowledge-preservation proof](https://corvidlabs.github.io/spec-sync/docs/comparisons/adversarial-proof/)

## Supported languages

SpecSync auto-detects source files and public exports across 33 languages:

TypeScript/JavaScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML, C, C++, Scala, Crystal, Nim, Erlang, Elixir, Perl, Common Lisp, Scheme, Emacs Lisp, Haskell, Lua, R, OCaml, Groovy, F#, Clojure, D, Objective-C, Bash, PowerShell, and Vala.

See the [language reference](https://corvidlabs.github.io/spec-sync/languages/) for per-language export detection and test exclusions.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), run the relevant tests, and keep specs synchronized with public behavior.

Security issues should follow [SECURITY.md](SECURITY.md), not a public issue.

## License

MIT — see [LICENSE](LICENSE).
