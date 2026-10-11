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
