---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: docs
---

# Docs

## Position to state

**Windows is not a supported target as of SpecSync 6.0. Run SpecSync under WSL.**

Stated plainly, not as a deprecation with an implied restoration. Nothing in the repository
promises a future Windows binary, so nothing may imply one. Where a reader needs an
alternative, WSL is named; `cargo install specsync` also still builds on Windows, and that is
worth saying where it fits, because the crate is not being made Windows-hostile — only the
prebuilt executable ends.

## Pages changed

| Page | Change |
|---|---|
| `README.md` "Pre-built binaries" | "macOS, Linux, or Windows" -> Linux and macOS, plus the WSL note |
| `github-action.md` "Available Binaries" | Remove the `specsync-windows-x86_64.exe` row |
| `github-action.md` floating-ref paragraph | Smoke tests are Linux and macOS |
| `github-action.md` "Multi-Platform Matrix" | Drop `windows-latest` from the example; a reader copying it would otherwise get a hard failure |
| `quickstart.md` | Qualify "the binary for your platform" |
| `adversarial-proof.md` | "Linux, macOS, Windows" -> the platforms actually shipped |

## Pages deliberately not changed

- `site/src/content/docs/mcp-security.md` — Windows junction, backslash and quarantine
  paragraphs are boundary correctness for content and code that both still exist.
- `docs/ci-confidence.md`, `fledge.toml`, `AGENTS.md`, `ci.yml:277` — these describe which
  *runners* CI uses. The `qualify` lane still runs on Windows, so all of them remain accurate.
- `CHANGELOG.md` history and `.specsync/archive/**` — released fact and digest-bound
  evidence. The removal is recorded as a new `[Unreleased]` entry instead.
- The "PowerShell" entries in the supported source-language lists in `README.md` and
  `site/.../index.md` — that is a language SpecSync parses exports from, not a platform it runs on.
