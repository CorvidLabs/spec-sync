---
spec: manifest.spec.md
---

## Key Decisions

- **Process-free parsing with structural Cargo security checks**: General metadata extraction remains string/regex based. MCP snapshot and confinement discovery parse bounded Cargo manifests with the in-process `toml` crate; no manifest discovery launches a process.
- **Fixed parse order, merged results**: Parsers run in a fixed sequence; results are merged with first-module-name-wins on conflict. This keeps behavior deterministic across runs.
- **Checked Gradle failure**: Missing ordinary manifests remain a normal skip. Present unreadable or malformed Gradle settings return `Err` from `discover_from_manifests_checked`; the checked path merges the exact single-read parse result, while the compatibility wrapper collapses failure to an empty result.
- **Settings are independently authoritative**: A root `settings.gradle[.kts]` is sufficient to
  discover a Gradle multi-project workspace. Its modules and parse failures are never hidden by a
  missing root `build.gradle[.kts]`.
- **Workspace/monorepo support**: Cargo workspaces, package.json workspaces, and Gradle multi-module projects are handled. Cargo security preflight uses real TOML; Gradle settings use one comment/escape-aware parser for Groovy/Kotlin include forms and effective `projectDir` mappings so MCP preflight and general discovery agree.
- **Fail-closed Gradle subset**: Include arguments must be complete string literals. Supported
  assignment-style `projectDir` and method-style `setProjectDir` expressions are exactly
  `file(<literal>)` and `new File(rootDir, <literal>)` with no trailing expression; dynamic
  arguments, unescaped double-quoted interpolation, alternate bases, extra arguments, and
  unsupported mutators reject the parse. Escaped/single-quoted dollars stay literal, and
  Unicode/octal escapes are decoded before path confinement.
- **Executable-directive subset**: Triple-quoted documentation and nested comments are masked
  before parsing. Include and project-directory mutations must use the supported top-level direct
  syntax; aliases, qualification, conditionals, closure/block scope, spacing tricks, compound
  assignment, and concatenated `newFile` lookalikes fail closed.
- **Raw identity validation precedes colon mapping**: Drive-qualified, rooted, UNC, and
  parent-escaping include identities and project selectors reject before Gradle `:` separators are
  converted to `/`; ordinary nested Gradle identities remain compatible.
- **Capability-confined Gradle probing**: Effective Gradle directories are walked component by
  component through a retained project-root capability with no-follow metadata. Symlinks and
  Windows reparse points fail checked discovery before source probing or traversal.
- **Capability-confined Gradle manifests**: Present Gradle build/settings manifests are selected
  through that retained capability. Every present filename variant is preflighted before
  precedence selection, rejected when linked/reparse-backed, non-regular, or identity-unstable,
  bounded to 4 MiB, and parsed from retained bytes instead of an ambient path read.
- **Locally governed Gradle directives**: Unsupported inclusion APIs and indirect or conditional
  include/project-directory mutations fail closed, while unrelated top-level control flow and
  identifier/documentation uses remain compatible.
- **One retained checked authority**: Caller-retained checked discovery uses the retained project
  capability for Cargo, Swift, Node, Dart, Go, Python, and Gradle manifests plus nested workspace
  directories. Non-Gradle retained reads are no-follow, non-blocking, identity-continuous, UTF-8
  checked, and deterministically bounded; only compatibility wrappers retain ambient best effort.
- **Bounded workspace graph**: Cargo member declarations and Node workspace patterns are charged as
  work before expansion. Normalized workspace nodes are deduplicated and completed discoveries are
  memoized, preventing duplicate declarations from replaying cached subtrees exponentially.
- **Swift test target exclusion**: `.testTarget()` entries are explicitly skipped to avoid polluting the module list with test infrastructure.
- **Python priority**: `[project]` section is checked before `[tool.poetry]` in pyproject.toml, reflecting the ecosystem's migration toward PEP 621.

## Files to Read First

- `src/manifest.rs` — Single-file module with a parser function per language ecosystem.

## Current Status

Implemented for all 7 manifest formats. Gate callers use `discover_from_manifests_checked()` so
malformed Gradle discovery is inconclusive instead of a compatibility fallback. The CHG-0063
independent-review contract additionally requires raw module validation before colon mapping,
literal assignment/method project-directory parsing, and retained no-follow component confinement.
Gradle build/settings selection and reads are likewise bounded and capability-confined; unescaped
double-quoted interpolation rejects and encoded path escapes are decoded before confinement.
The final parser amendment also rejects indirect executable mutations and drive-relative raw
identities while preserving rooted nested Gradle names. The post-review amendment preflights
shadowed Gradle filename variants, binds native file identity across open/read, rejects invoked
unsupported inclusion APIs, and narrows control-flow rejection to governed directives.
The acceptance-remediation pass extends retained authority to every recognized manifest parser and
nested workspace probe, eliminating the ambient swap-read-restore interval.
The latest remediation bounds and deduplicates Cargo/Node workspace expansion, memoizes completed
nodes, and verifies that nested manifest/workspace directories remain reachable from the retained
project root after enumeration and around reads. The reported focused manifest run passed 41
tests. Fresh independent rereview, the full post-remediation suite, hosted-Windows runtime,
repository/CI, trust, and provenance evidence remain pending. MCP Cargo workspace paths come from
validated TOML values.

## Notes

- Balanced parenthesis extraction is used for Swift's `Package.swift` — it's a mini expression parser for the `.target(name: ..., path: ...)` syntax.
- Go module detection uses the last path segment of the module name (e.g., `github.com/user/repo` → `repo`) and probes for standard directory conventions (cmd, internal, pkg, api).
