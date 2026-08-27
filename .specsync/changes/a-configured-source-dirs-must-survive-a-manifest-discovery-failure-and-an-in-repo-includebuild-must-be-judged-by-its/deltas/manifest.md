## MODIFIED

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
    `includeBuild` is decided by its argument rather than its token: one complete literal path
    confined beneath the project root parses and contributes no module, while an escaping,
    interpolated, dynamic, multi-argument, or trailing-expression argument fails closed. A guard
    that reads only the token cannot distinguish an ordinary in-repo composite build from one that
    leaves the repository, and refusing both makes a valid project unmeasurable. Its position is
    likewise not judged — a conditional or block-scoped `includeBuild` is accepted where the same
    shape of `include` is refused, because a composite build contributes no module whether or not
    its branch runs, so where it sits cannot change what is discovered. One-line and multi-line
    spellings of the same conditional therefore reach the same verdict.
15. Every recognized checked manifest ecosystem and nested workspace probe uses the caller's
    retained project capability with deterministic byte, entry, depth, UTF-8, link, special-file,
    and identity enforcement; ambient paths are only a final replacement diagnostic.
16. Cargo/Node workspace expansion charges declarations independently of unique retained bytes,
    deduplicates normalized nodes, and reuses completed results.
17. Retained nested manifest/workspace parents are reverified through the project root around
    enumeration and reads.
18. Single-project Gradle (no `include`) names the module from a literal `rootProject.name`
    assignment or, when that is unset, the project directory name — Gradle's own default.
    Package path segments under `src/main/{kotlin,java,scala}` are not modules.
19. Node workspace enumeration records child identities, opens children sequentially through the
    retained workspace base, and consumes child manifests/source probes only from identity-matching
    capabilities. Each verified base listing is released before the next distinct base, so
    swap/read/restore cannot mix generations and neither sibling nor base breadth exhausts handles.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| Manifest file missing | Parser returns `None`, skipped silently |
| Manifest file unreadable | Parser returns `None` (fs::read_to_string fails gracefully) |
| Non-Gradle manifest is unsafe, replaced, invalid UTF-8, over 8 MiB, or retained discovery exceeds 64 MiB/100,000 entries/256 components | Caller-retained checked discovery returns `Err` without ambient fallback or partial discovery; compatibility discovery returns an empty result |
| Cargo/Node workspace declarations exceed the expansion budget or repeat a completed normalized node | Checked discovery returns `Err` on budget exhaustion; otherwise it reuses the completed result without reparsing the subtree |
| Nested manifest/workspace parent is detached or replaced during enumeration/read | Caller-retained checked discovery returns `Err` after project-root reachability verification; detached and replacement generations are not mixed |
| Enumerated Node workspace is swapped during a child read, or sibling/base breadth exceeds the process descriptor limit | Child bytes/probes come from an identity-matching retained capability opened sequentially; completed base listings are released, so replacement generations are not mixed and handles remain bounded |
| Malformed non-Gradle manifest content | Best-effort extraction; missing fields result in defaults or skipped entries |
| Linked, reparse-backed, non-regular, replaced, oversized, unreadable, or invalid-UTF-8 Gradle build/settings manifest, including a shadowed filename variant | Checked discovery returns `Err` without reading a link referent or returning partial discovery; compatibility discovery returns an empty result |
| Malformed or dynamic Gradle include, invoked unsupported inclusion API, unescaped double-quoted interpolation, unsupported assignment/method project-directory form, rooted/drive/UNC/parent-escaping raw module identity or decoded effective path, or broken comments/escapes/parentheses | Checked discovery returns `Err`; compatibility discovery returns an empty result and gates stay inconclusive |
| Gradle-derived directory contains a symlink or Windows reparse-point component | Checked discovery returns `Err` before source probing/traversal; compatibility discovery returns an empty result and gates stay inconclusive |
| `includeBuild` names one literal path beneath the project root | Parses; the composite build contributes no module and the root build's `include(...)` list is unaffected |
| `includeBuild` escapes the project root, or its argument is not one complete literal (interpolated, dynamic, multiple, or followed by a configuration block) | Checked discovery returns `Err` naming the argument, not the token; compatibility discovery returns an empty result |
| Workspace member directory doesn't exist | Skipped (Cargo.toml existence check) |
| No parsers produce results | Returns default empty `ManifestDiscovery` |
