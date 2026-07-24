## MODIFIED

### REQUIREMENT REQ-manifest-001

Manifest discovery SHALL identify supported project modules and source roots deterministically
without claiming unsupported workspace expansion.

Acceptance Criteria

- `discover_from_manifests_checked` surfaces malformed Gradle settings to coverage and generation
  gates rather than returning partial discovery, and merges the exact parse from that same read
  instead of validating then rereading a mutable path.
- One parser handles Groovy/Kotlin comments, escapes, literal multiline includes, nested colon
  names, assignment-style `.projectDir = ...`, and method-style `.setProjectDir(...)`.
- Raw include identities and raw `project(...)` selectors are checked before Gradle colon notation
  is mapped to path separators; drive-qualified, rooted, UNC, and parent-escaping spellings reject
  while valid explicitly rooted nested identities remain supported. Unrooted drive-relative
  spellings such as `C:member` reject before colon mapping.
- Assignment and method project-directory values accept exactly `file(<literal>)` or
  whitespace-delimited `new File(rootDir, <literal>)`; concatenated/dynamic lookalikes such as
  `newFile(...)` are unsupported and fail closed.
- Dynamic include arguments, alternate `new File` bases, extra arguments, and trailing assignment
  or method expressions fail checked discovery without returning partial modules.
- Include and project-directory directives are recognized only as top-level executable statements:
  directives inside single-, double-, or triple-quoted strings and nested block/line comments are
  inert, while qualified, aliased, conditional/block-scoped, compound-assignment, and otherwise
  indirect mutations fail closed. Unsupported multiline literals used as directive arguments fail
  rather than disappearing into an empty include.
- Double-quoted `$name` and `${expression}` interpolation fails checked discovery, including when
  Unicode or octal escape decoding reconstructs the dollar. Explicit `\$` and Groovy
  single-quoted dollar literals remain compatible.
- Gradle module identities and `projectDir` literals are confined to project-relative paths:
  rooted, drive-qualified, UNC, and parent-underflow forms fail before source probing, while safe
  literal spellings retain compatibility.
- General module discovery and MCP snapshot preflight use the same effective Gradle module paths.
- Every component of a Gradle-derived effective directory is checked no-follow through a retained
  project-root capability before source probing/traversal; Unix symlink and Windows reparse-point
  components fail checked discovery without reading their referents.
- Present Gradle build/settings manifests are read as regular non-link entries through the retained
  root capability, capped at 4 MiB, and rejected when linked, reparse-backed, non-regular,
  oversized, unreadable, invalid UTF-8, or changed in type during acquisition.
- Every present Gradle build/settings filename is preflighted before precedence selection and its
  native path identity must match the opened handle before and after the bounded read.
- Invoked unsupported inclusion APIs such as `includeFlat` and `includeBuild` fail checked
  discovery, while unrelated control flow remains compatible unless it governs an unsupported
  include/project-directory mutation.
- A present `settings.gradle[.kts]` is parsed and validated even when no root
  `build.gradle[.kts]` exists.
- MCP Cargo workspace snapshot and confinement discovery parse bounded manifests as real TOML.
- Malformed MCP Cargo TOML/workspace shapes make MCP operations inconclusive; malformed Gradle
  declarations make checked coverage gates inconclusive without partial module results.
- Caller-retained checked discovery acquires Cargo, Swift, Node, Dart, Go, and Python manifests,
  nested Cargo workspace manifests, workspace entries, and source-directory probes through the
  retained project capability. Reads are no-follow, non-blocking, identity-continuous, strict
  UTF-8, and bounded to 8 MiB each/64 MiB cumulatively; directory discovery is sorted and bounded
  to 100,000 entries/256 components. Ambient parser reads are forbidden.
- Every declared Cargo workspace member and Node workspace pattern consumes a deterministic
  expansion-work entry before traversal. Normalized workspace nodes are deduplicated and completed
  results are memoized; budget exhaustion returns an inconclusive checked error without partial
  modules or repeated subtree parsing.
- Nested manifest/workspace directories remain reachable through the retained project root after
  enumeration and before/after reads; detached-parent replacement fails without mixing
  generations.

### SPEC SECTION Public API

#### Exported Constants

| Constant | Type | Description |
|----------|------|-------------|
| `MAX_GRADLE_MANIFEST_BYTES` | `u64` | Crate-visible 4 MiB ceiling shared by retained Gradle manifest readers |

#### Exported Structs

| Struct | Fields | Description |
|--------|--------|-------------|
| `ManifestModule` | `name: String`, `source_paths: Vec<String>`, `dependencies: Vec<String>` | A module/target discovered from a manifest file |
| `ManifestDiscovery` | `modules: HashMap<String, ManifestModule>`, `source_dirs: Vec<String>` | Aggregated result of parsing all manifest files in a project |
| `GradleSettingsModule` | `name: String`, `path: String` | Crate-visible normalized Gradle module identity and effective project directory |

#### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `discover_from_manifests` | `root: &Path` | `ManifestDiscovery` | Compatibility discovery that returns an empty result when checked discovery is malformed |
| `discover_from_manifests_checked` | `root: &Path` | `Result<ManifestDiscovery, String>` | Discover modules while surfacing unreadable or malformed Gradle settings to gate callers |
| `discover_from_manifests_checked_with_root` | `root: &Path, project_root: &Dir` | `Result<ManifestDiscovery, String>` | Crate-visible checked discovery that reuses a caller-retained project-root capability and rejects an ambient/retained root identity mismatch |
| `parse_gradle_settings` | `content: &str` | `Result<Vec<GradleSettingsModule>, String>` | Crate-visible shared parser for Groovy/Kotlin includes plus assignment-style and method-style literal project-directory overrides |

### SPEC SECTION Invariants

1. Gradle settings parsing is comment- and escape-aware and supports literal Groovy/Kotlin multiline
   include declarations.
2. Raw include identities and project selectors reject drive-qualified, rooted, UNC, and
   parent-escaping forms before colon-to-path conversion.
3. Nested colon names and the supported literal assignment/method project-directory forms resolve
   to one deterministic effective project-relative directory per module.
4. Dynamic, qualified, aliased, conditional/block-scoped, or otherwise indirect includes and
   `projectDir`/`setProjectDir` mutations, unsupported bases/arity/suffixes, compound assignments,
   and unsupported multiline directive arguments fail without partial discovery.
5. Double-quoted Gradle interpolation is rejected after escape decoding; explicit escaped-dollar
   and Groovy single-quoted literal-dollar forms remain deterministic literals.
6. Checked discovery reports malformed Gradle input; compatibility discovery may return an empty
   result, but gate callers remain inconclusive.
7. Checked Gradle discovery merges the exact single-read parse result.
8. MCP Cargo workspace discovery trusts only structurally parsed TOML members and target paths.
9. Settings-only Gradle workspaces discover included modules, while malformed settings remain an
   inconclusive checked-discovery error.
10. Gradle module and effective project-directory paths cannot traverse above the project root or
   select rooted, drive-qualified, or UNC locations; rejection occurs before partial discovery or
   filesystem probing.
11. Every Gradle-derived directory component is inspected no-follow through the retained root
    capability; symlink and reparse-point components reject before source probing or traversal.
12. Present Gradle build/settings manifests are bounded regular non-link retained-capability reads;
    malformed endpoints or bytes reject before partial discovery.
13. Every present filename variant is preflighted before precedence and remains identity-stable
    through open/read; unsafe shadowed variants cannot evade checked discovery.
14. Unsupported invoked inclusion APIs and governed indirect/conditional mutations fail closed,
    while unrelated Gradle control flow and identifier/documentation uses remain compatible.
15. Every recognized checked manifest ecosystem and nested workspace probe uses the caller's
    retained project capability with deterministic byte, entry, depth, UTF-8, link, special-file,
    and identity enforcement; ambient paths are only a final replacement diagnostic.
16. Cargo/Node workspace expansion charges declarations independently of unique retained bytes,
    deduplicates normalized nodes, and reuses completed results.
17. Retained nested manifest/workspace parents are reverified through the project root around
    enumeration and reads.
18. Node workspace enumeration records child identities, opens children sequentially through the
    retained workspace base, and consumes child manifests/source probes only from identity-matching
    capabilities so swap/read/restore cannot mix generations or exhaust handles by sibling count.
