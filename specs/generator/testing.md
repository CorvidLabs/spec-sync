---
spec: generator.spec.md
---

## Regression Matrix

| Case | Required Result |
|------|-----------------|
| Uncovered module | Spec and companions created |
| Existing spec | No overwrite |
| Custom template | Used with per-file fallback, including retained-root generation for configured modules |
| Inference env vars set | Output remains deterministic; no command/network execution |
| Generation outcome | Count and relative paths only |
| Root retarget after coverage | Default and batch JSON generation fail inconclusively; original and replacement bytes remain unchanged |
| Retained destination path | Absolute/rooted/prefix/parent components reject before a write |

## Cargo generation regressions (#813)

| Case | Setup | Required Behavior |
|------|-------|-------------------|
| Fresh Cargo library | `init` then text/JSON/batch `generate` | Creates demo spec bound to src/lib.rs with add and companions |
| Cargo binary or combined roots | Initialized crate | All discovered root entry points are bound |
| Cargo workspace | Two member crates | Each generated spec owns only its member source; source containers do not get duplicate owners |
| Existing and named owners | Cargo root, existing core spec, directory/flat modules | Existing spec bytes are preserved; root fallback excludes more specific bindings |
| Captured manifest mapping | Change Cargo.toml after coverage | Library generation entry points use the captured mapping without rediscovery |

Run `cargo test --test integration generate_cargo` and `cargo test generator::`. Preserve malformed-manifest, missing-module, and retained-root replacement regressions.

## Cargo target identity review regressions (#813)

Cover same-name directory and flat source modules, arbitrary explicit binary files, a target sharing its package name, nested target roots and workspace prefixes. Exact target ownership must not be duplicated by package or guessed filename specs.

## Additional review regressions (#813)

| Case | Required behavior |
|------|-------------------|
| Explicit mapping has no measured files | Remain skipped; never fall back to a crate root or conventional filename |
| Two targets share an exact file | One deterministic owner; batch selection wins an equal-root tie |
| Cargo and npm share a source root | One owner per file and complete post-generation coverage |
| More specific unselected target | Batch generation cannot capture its source through a broader root |
| Pathless Cargo binary | Flat and nested main.rs defaults remain covered, including an existing differently named owner |

The four added process regressions failed on 5b640b7 before implementation. All eleven Cargo process tests, 51 generator unit tests and 61 manifest unit tests pass after the corrections.

Preserve `mcp_multiline_cargo_workspace_members_survive_coverage_and_generation`: an existing empty member spec does not block named generation for uncovered member sources. The first full verification caught this boundary; directory assignment is limited to selected manifest modules while unselected exact targets retain precedence.

## 6.1.1 release corrections

`scoped_module_generation_entry_points_preserve_identity_and_leaf_paths` covers ambient entry points, valid YAML, built-in/custom companion references and context-scaffold identity. Process tests cover default, JSON and batch output, coverage, reruns and inconclusive output errors.

## Exact-file creation follow-up

Run `cargo test --locked --offline --bin specsync creation_source_selection` for exact root files, flat-name matching, explicit mapping precedence including missing mappings, file/directory overlap, distinct-file ambiguity, configured extension/extensionless/test/exclusion rules, escaped mappings, symlink leaves/parents and Unix named pipes. `creation_source_selection_keeps_directory_priorities_and_explicit_dot` retains each command's prior matching order; root-alias coverage includes `.`, `./`, `./.` and `.//` with exact-file overlap.

The preserved f72 optimized CLI produced `files: []` after init then new/scaffold for root lib.rs; subsequent ordinary check failed, while src/lib.rs controls bound their file. Root owns the real process regressions for all three creation commands and final full qualification. Passing draft checks do not represent authored prose or human implementation acceptance.

The root-owned process regressions are `create_commands_bind_exact_cargo_sources_without_widening_roots` and `creation_commands_keep_two_distinct_exact_sources_ambiguous`; they remain separate from the inline selector tests.
