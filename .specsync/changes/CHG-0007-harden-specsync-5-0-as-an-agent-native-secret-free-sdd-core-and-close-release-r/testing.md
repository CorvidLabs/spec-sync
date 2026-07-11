---
change: CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-exports-001 | Regex/AST parity covers valid visibility spacing, private inline modules, declarations, and re-exports; a strict multi-file fixture is modeled on Fledge modules |
| REQ-ai-002 | Dependency-tree/source scan plus command-nonexecution and value-safe legacy configuration regressions |
| REQ-change-015 | Quiet lifecycle checking integration proves commands still execute while comment stdout stays protocol-clean |
| REQ-cli-002 | CLI regressions reject retired provider/model flags while Agents and MCP remain available |
| REQ-cli-args-002 | Clap parser and integration regressions cover deterministic Generate selection and retired-flag rejection |
| REQ-cmd-check-002 | Full `--fix` suite plus `SPECSYNC_AI_COMMAND` sentinel regression proves local deterministic repair |
| REQ-cmd-comment-003 | Comment integration proves child output suppression, failure preservation, and CI-safe transport |
| REQ-cmd-generate-001 | Generate all/batch/JSON regressions prove deterministic output and no inference environment effect |
| REQ-comment-001 | UTF-8 boundary unit test proves rendered markdown remains at or below 49,152 bytes with guidance |
| REQ-config-001 | JSON/TOML/local legacy-key tests prove values are ignored, never retained, and never echoed |
| REQ-generator-001 | Generator unit/integration suite covers templates, companions, discovery, and no-overwrite behavior |
| REQ-mcp-001 | MCP unit/integration tests prove deterministic generation and value-safe rejection of retired arguments |
| REQ-types-001 | Compile/source/dependency checks prove the provider enum and inference fields are absent |

Release evidence also covers the higher-level REQ-core-001 and REQ-release-security-001 scope: 1,512 unit and 187 integration tests, rustfmt, Clippy with warnings denied, RustSec, 100% scored canonical specs, Astro 6.4.8, 23 site tests, zero Astro diagnostics, a 34-page build, and a final `bun audit` with no vulnerabilities after compatible sanitizer and transitive security updates.

The final gate includes all Rust unit/integration tests, Clippy with warnings denied, rustfmt, strict 100% spec coverage, score, optimized build, documentation and VSIX builds, packaged Action consumer, single/concurrent/five-epic SDD examples, and a clean target/artifact check.

PR #335's final pre-accept matrix passed on Linux, macOS, and Windows, with CodeQL for Rust, Actions, and JavaScript/TypeScript; packaged Action consumption; RustSec; coverage; site; VSIX; workflow validation; spec-check; and bounded `corvid-pet` reporting all green.
