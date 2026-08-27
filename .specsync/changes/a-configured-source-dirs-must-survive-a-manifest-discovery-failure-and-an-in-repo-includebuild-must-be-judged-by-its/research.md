---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: research
---

# Research

## What the manifest is actually used for in checked coverage

This is what decides whether degrading is safe, so it was established before anything was changed.
`compute_coverage_checked` binds the discovery result once and reads it in exactly two places:

- seeding `candidate_directories` from `module.source_paths`;
- naming manifest-declared modules with no spec, into `unspecced_modules`.

Both are module ATTRIBUTION. The file list, the LOC totals, and every percentage come from
`config.source_dirs` by way of `select_coverage_source_directories`. So when the source list was
STATED, a lost manifest costs module names and nothing else — and when it was NOT stated, the source
list is itself discovery output and losing the manifest means there is nothing trustworthy to
measure. That asymmetry is the whole fix.

## How the same failure is handled elsewhere

| site | on discovery error |
|---|---|
| `config.rs:66` `detect_source_dirs` | `unwrap_or_else` → scan |
| `validator.rs:430` `retained_source_dirs` | `Err(_)` → `retained_source_dirs_by_scan` |
| `validator.rs` `compute_coverage` (compatibility) | `unwrap_or_else` → inconclusive report |
| **`compute_coverage_checked`** | **`?` → aborts the command** |

Coverage was the outlier, and it is the only one on the path CI depends on.

## The principle was already written down

`specs/validator/context.md` states it as a key decision:

> **Retained zero-config discovery**: Configuration fallback and manifest/source autodetection run
> only when `source_dirs` is omitted.

`retained_config` honours it — `retained_explicit_source_dirs_skip_unrelated_manifest_autodetection`
is the guarding test. `compute_coverage_checked` then re-ran discovery unconditionally and let it
abort. The fix brings coverage in line with a rule the module had already adopted.

## Why the existing parser test could not have caught the rejection

`gradle_settings_reject_unsupported_include_prefixed_workspace_mutators` used `"../outside"` for
every one of its four fixtures. Since the verdict came from the token, the test passed whether the
parser read the path or ignored it entirely — it could not fail for the right reason. There was no
fixture anywhere in the repository for an in-repo composite build, which is why the shape survived
rc.1 through rc.7: it does not exist in this repository or in any repository tested against.

## Gradle semantics checked before touching the sibling mutators

- `includeBuild(path)` resolves relative to the ROOT project directory, so an in-repo argument is an
  ordinary, correct composite build.
- `includeFlat(name)` resolves to `../name` — the PARENT of the root — so its argument is outside
  the project by construction and no argument makes it supportable.
- `includeWorkspace` is not a form this parser models.

Only `includeBuild` therefore gained argument-based judgment; the other two stay in the token arm,
with the reason recorded in the code and the test rather than left to be rediscovered.

## Verification surface

`fledge.toml` puts `lint` (`cargo clippy -- -D warnings`) in the `verify` and `ci` lanes only. It is
not among the commands `change check` runs, so `change check` can be green on a tree CI will block.
Clippy was run by hand, bare rather than `--all-targets`, matching the lane definition.
