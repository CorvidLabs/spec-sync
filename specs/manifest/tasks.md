---
spec: manifest.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Add CMakeLists.txt support for C/C++ projects
- [ ] Add .csproj/.sln support for C# projects
- [ ] Handle Cargo workspace `members` with glob patterns (e.g., `"crates/*"`)
- [ ] Extract dependency information from manifests for cross-module `depends_on` pre-population

## Done

- [x] Cargo.toml parser (packages, binaries, workspace members)
- [x] Package.swift parser (targets with balanced paren extraction)
- [x] build.gradle/build.gradle.kts parser (modules, Android detection)
- [x] Shared Gradle settings parser for Groovy/Kotlin comments, escapes, quoting, multiline includes, nested names, and `projectDir` overrides
- [x] Checked Gradle discovery that keeps malformed inputs inconclusive for coverage gates
- [x] Reject dynamic Gradle includes and unsupported `projectDir` bases, arguments, or suffixes without partial discovery
- [x] Reject rooted, drive-qualified, UNC, and parent-escaping Gradle module and `projectDir` paths
  before shared CLI discovery can inspect outside the project
- [x] Discover and validate settings-only Gradle multi-project workspaces without requiring a root build script
- [x] Real-TOML Cargo workspace discovery for MCP snapshot and confinement preflight
- [x] package.json parser (workspaces, monorepo support)
- [x] pubspec.yaml parser (single-entry lib/)
- [x] go.mod parser (module name + standard dirs)
- [x] pyproject.toml parser (project and poetry support)

## CHG-0063 Independent-Review Amendment

- [x] Amend the canonical contract for raw drive-prefix validation, literal `setProjectDir`
  support, and retained no-follow Gradle directory confinement.
- [x] Verify focused parser coverage for drive-qualified raw include and project-selector values.
- [x] Verify literal `setProjectDir(file(...))` and `setProjectDir(new File(rootDir, ...))` support,
  plus dynamic, unsupported, traversal, drive, and UNC rejection.
- [x] Verify Unix symlink rejection before outside source probing or traversal.
- [x] Reject unescaped double-quoted Gradle interpolation while preserving escaped and
  single-quoted literal dollars; decode Unicode/octal escapes before path confinement.
- [x] Reject linked/reparse-backed, non-regular, oversized, unreadable, and invalid-UTF-8 Gradle
  build/settings manifests through bounded retained-capability reads.
- [x] Reject indirect, qualified, conditional, block-scoped, whitespace-separated, and compound
  Gradle project-directory mutations; ignore triple-quoted documentation and nested comments.
- [x] Require the whitespace-delimited `new File` constructor and reject unrooted drive-relative
  identities while preserving genuine rooted nested Gradle identities.
- [x] Preflight all four Gradle build/settings filenames before precedence selection so an unsafe
  shadowed variant cannot evade checked discovery.
- [x] Bind each Gradle manifest's native filesystem identity before open, after open, and after the
  bounded retained read on Unix and Windows.
- [x] Scope control-flow rejection to governed include/project-directory directives so unrelated
  valid Gradle logic remains compatible.
- [x] Reject invoked unsupported inclusion APIs such as `includeFlat` and `includeBuild` while
  leaving ordinary identifiers and documentation inert.
- [x] Route Cargo, Swift, Node, Dart, Go, and Python checked discovery plus nested workspace probes
  through one bounded retained project capability without ambient parser reads.
- [x] Bound every declared Cargo member and Node workspace pattern, deduplicate normalized workspace
  nodes, and memoize completed results so repeated declarations cannot amplify parsing work.
- [x] Reverify nested manifest/workspace parent reachability after traversal and around retained
  reads; reject detached-parent replacement races.
- [x] Record enumerated Node workspace child identities and open each sequentially through the
  retained base capability for manifest reads/source probes, with swap/read/restore and
  descriptor-bound regressions.
- [x] Add limit/limit-plus-one, duplicate Cargo/Node expansion, linked duplicate-member, and
  replaced retained-parent regressions; the combined focused manifest run passes 51 tests.
- [ ] Add remaining invalid-UTF-8 and after-open/read table-driven retained-manifest regressions
  across every supported non-Gradle ecosystem.
- [ ] Verify hosted-Windows junction/reparse-point rejection before outside source probing or
  traversal.
- [ ] Obtain fresh exact-tree independent reviews, full repository/CI, trust, and Attest evidence.

## Gaps

- No support for Bazel BUILD files or Meson build definitions
- Workspace glob expansion not implemented for Cargo `members` (literal paths only; `package.json` workspace globs like `packages/*` are supported, Cargo's are not)
- Dependency extraction is only wired up for Cargo (`[dependencies]`) and Swift target `dependencies:`; other manifests leave `dependencies` empty

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
