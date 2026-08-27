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
- **A directive is judged by its argument when the argument is what makes it safe**: `includeBuild`
  was refused on the token prefix alone, so an ordinary in-repo composite build
  (`includeBuild("vendor/shared")`) failed identically to one escaping the repository
  (`includeBuild("../outside")`). It is now decided by the same literal-only parsing and path
  confinement `include(...)` already uses; an accepted composite build contributes no module.
  `includeFlat` and `includeWorkspace` stay token-refused for a stated reason — `includeFlat`
  resolves against the PARENT of the root, so its argument is outside the project by construction,
  and `includeWorkspace` is not a form this parser models. Its POSITION is not judged either: a
  conditional or block-scoped `includeBuild` is accepted where the same shape of `include` is
  refused, because a composite build contributes no module whether or not its branch runs. Getting
  that half right mattered — the first cut accepted the three-line conditional and refused the
  one-line one, since only there did the closing `}` land on the declaration's own line.
- **A configuration block is not a project declaration** (#725): `includeBuild(path) {
  dependencySubstitution { … } }` is the NORMAL spelling — substituting a local project for a
  published coordinate is the reason to have a composite build — and #723 deliberately kept refusing
  it, leaving the parser accepting the minority bare form and rejecting the common one. The block
  holds substitution rules; it declares no project and no source directory, so a BALANCED block is
  now skipped whole. What makes that safe is that skipping is confined to finding where the
  declaration ends: the path is parsed and confined from inside the parentheses in front of the
  block, so `includeBuild("../outside") { … }` still fails on the path; the block's text is never
  removed from `content`, so the `include`, `projectDir`, and `project(...)` guards still walk it;
  the brace scan is quote-aware and runs after comment stripping, so a brace in a string or a
  comment moves nothing; and an unbalanced block is refused, because its extent is precisely what is
  unknown. Even a mis-scanned extent could only drop trailing text from the verdict, never an
  argument from it.
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
project root after enumeration and around reads. Structural Cargo/Node parsing and retained
directory-listing continuity now close the latest independent-review gaps. Node workspace child
identities are recorded during enumeration, then each child is opened sequentially through the
retained workspace-base capability for manifest reads and source probes. A swap/read/restore
interval cannot mix generations. Each completed base listing is reachability-verified and released,
so broad sibling sets and broad distinct-base sets do not exhaust directory handles. Fresh
combined results pass 52 focused manifest tests and 1,953 unit plus 312 integration tests. Fresh
independent rereview, hosted-Windows runtime, repository/CI, trust, and provenance evidence remain
pending. MCP Cargo workspace paths come from validated TOML values.

## Notes

- Balanced parenthesis extraction is used for Swift's `Package.swift` — it's a mini expression parser for the `.target(name: ..., path: ...)` syntax.
- Go module detection uses the last path segment of the module name (e.g., `github.com/user/repo` → `repo`) and probes for standard directory conventions (cmd, internal, pkg, api).

## Lesson (#723)

A guard written against one shape catches every shape that shares its token. `includeBuild` was
refused by prefix, and every fixture in the guarding test used `"../outside"` — so the test passed
whether the parser read the path or ignored it entirely. It could not fail for the right reason, and
the case it silently covered (an ordinary in-repo composite build, which does not exist in this
repository or in any repository tested against) blocked a real adopter on every 6.0 candidate from
rc.1 to rc.7. When a rejection is decided by a token, ask what the ARGUMENT would have said, and add
the accepted fixture before trusting the refusing one.

## Lesson (#725)

#723 fixed the token-vs-argument bug and then drew the line one notch short of the reported form:
the path was read, but a trailing configuration block was still refused — so the parser accepted
`includeBuild("vendor/shared")` and rejected `includeBuild("vendor/shared") { … }`, which is the
spelling almost every composite build actually uses. Asking "which spelling is the common one?"
would have caught it; the accepted fixture added for #723 used the rare one, so nothing failed.

The reason it was a follow-up issue rather than a second outage is worth keeping. #723 also fixed
the PRECEDENCE rule that let a discovery failure veto a stated `source_dirs`, and that class-level
fix caught this un-anticipated instance: the adopter got a notice naming the file, the reason, the
fallback, and the consequence, instead of a blocked run. When a parser fix and a precedence fix are
both available, the precedence fix is worth more, because it contains the instances you have not
thought of yet.
