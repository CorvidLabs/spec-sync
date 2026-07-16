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

[Quick start](#quick-start) · [Full SDD lifecycle](#full-sdd-lifecycle) · [Live documentation](https://corvidlabs.xyz/spec-sync/docs/) · [Examples](#executable-examples) · [Comparisons](#how-it-compares)

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

[Read the adversarial proof](site/src/content/docs/comparisons/adversarial-proof.md)

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

[Read the workflow guide](site/src/content/docs/workflow.md) or run the [complete lifecycle example](examples/sdd-lifecycle/).

## Install

### Cargo

```bash
cargo install specsync
```

### Homebrew

```bash
brew install CorvidLabs/tap/spec-sync
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
- uses: CorvidLabs/spec-sync@v5.1.1
  with:
    version: '5.1.1'
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

For an existing 4.x project, use the guided migration and adoption flow described in the [configuration](site/src/content/docs/configuration.md) and [workflow](site/src/content/docs/workflow.md) guides.

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

[Read the complete spec format](site/src/content/docs/spec-format.md), [companion-file reference](site/src/content/docs/companion-files.md), and [workflow conventions](site/src/content/docs/workflow.md).

## Documentation

| Start here | Reference and integration |
|---|---|
| [Why SpecSync?](site/src/content/docs/why-specsync.md) | [CLI reference](site/src/content/docs/cli.md) |
| [Quick start](site/src/content/docs/quickstart.md) | [Configuration](site/src/content/docs/configuration.md) |
| [Workflow guide](site/src/content/docs/workflow.md) | [Spec format](site/src/content/docs/spec-format.md) |
| [Companion files](site/src/content/docs/companion-files.md) | [Language registry](site/src/data/languages.json) |
| [Architecture](site/src/content/docs/architecture.md) | [Cross-project references](site/src/content/docs/cross-project-refs.md) |
| [AI and coding agents](site/src/content/docs/integrations/ai-agents.md) | [GitHub Action](site/src/content/docs/integrations/github-action.md) |
| [Live docs hub](https://corvidlabs.xyz/spec-sync/docs/) | [VS Code extension](site/src/content/docs/integrations/vscode-extension.md) |

## Executable examples

The examples create disposable projects and run the real CLI:

- [Complete SDD lifecycle](examples/sdd-lifecycle/)
- [Five evolving product epics](examples/sdd-five-epics/)
- [Ordered concurrent changes](examples/sdd-concurrent-changes/)
- [CI gate](site/src/content/examples/ci-gate.mdx)
- [Polyglot project](site/src/content/examples/polyglot.mdx)
- [Rust workspace](site/src/content/examples/rust-workspace.mdx)

## How it compares

SpecSync can stand alone or enforce the implementation layer beneath planning-oriented tools:

- [SpecSync vs. Spec Kit](site/src/content/docs/comparisons/spec-kit.md)
- [SpecSync vs. OpenSpec](site/src/content/docs/comparisons/openspec.md)
- [Use SpecSync, Spec Kit, and OpenSpec together](site/src/content/docs/comparisons/using-together.md)
- [Adversarial detection and knowledge-preservation proof](site/src/content/docs/comparisons/adversarial-proof.md)

## Supported languages

SpecSync auto-detects source files and public exports across 33 languages:

TypeScript/JavaScript, Rust, Go, Python, Swift, Kotlin, Java, C#, Dart, PHP, Ruby, YAML, C, C++, Scala, Crystal, Nim, Erlang, Elixir, Perl, Common Lisp, Scheme, Emacs Lisp, Haskell, Lua, R, OCaml, Groovy, F#, Clojure, D, Objective-C, Bash, PowerShell, and Vala.

See the [detailed language profiles](site/src/data/languages.json) for 12 representative stacks and the
[extractor source](src/exports/) for exact support, export detection, and test exclusions across all 33 families.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), run the relevant tests, and keep specs synchronized with public behavior.

Security issues should follow [SECURITY.md](SECURITY.md), not a public issue.

## License

MIT — see [LICENSE](LICENSE).
