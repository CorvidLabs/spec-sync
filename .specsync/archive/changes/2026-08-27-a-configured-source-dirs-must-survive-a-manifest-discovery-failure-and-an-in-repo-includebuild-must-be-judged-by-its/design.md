---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: design
---

# Design

## Why the precedence fix comes first

The parser gap and the precedence gap are independent, and only one of them removes a class.

Fixing the parser removes ONE rejection: `includeBuild`. The next manifest form this parser cannot
read — in Gradle or any other ecosystem — reopens the same hole, because the hole is not
`includeBuild`. It is that a failure to INFER something the user already STATED is allowed to veto
the statement and abort the command. Fixing the precedence closes that for every present and future
parser gap, and it unblocks the reported project without anyone having to agree on what composite
build support should mean.

So the precedence lands as the primary change and the parser follows it.

## R1 — precedence

`SpecSyncConfig` gains `source_dirs_set: bool`, `#[serde(skip)]`, following the existing
`enforcement_set` exactly. The loaders already compute this — `parse_json_config_with_source_dirs`
has `source_dirs_configured`, `parse_toml_config_with_source_dirs` has `has_source_dirs`, and the
retained loader has `retained_config_has_source_dirs` — they simply threw it away after using it to
decide the fallback. Each now records it.

The retained loader is the subtle one. It decides the source-list fallback with its OWN predicate,
separate from the parser's. If the flag were left to the parser while the override were decided
here, the two could disagree and coverage would treat a SCANNED list as a stated one — degrading a
manifest error over a guess, the one thing the flag exists to prevent. One predicate therefore sets
both.

`compute_coverage_checked`'s unconditional

```rust
let manifest = discover_from_manifests_checked_with_root(root, &project)?;
```

becomes `retained_coverage_manifest(root, &project, config)`, which returns
`(ManifestDiscovery, Vec<String>)`:

| `source_dirs` | discovery fails | result |
|---|---|---|
| omitted | yes | `Err` — unchanged. The list coverage would measure came from this discovery. |
| stated | yes | `Ok` over an empty discovery, plus one notice. |
| either | no | `Ok`, no notice — unchanged. |

The manifest is used at exactly two places downstream: seeding `candidate_directories`, and naming
manifest-declared modules that have no spec. Both are module ATTRIBUTION. File and LOC coverage come
entirely from `config.source_dirs`, which is why degrading is safe here and would not be if the
source list were the thing that had been inferred.

`CoverageReport` gains `manifest_notices: Vec<String>`, placed beside `missing_files` and
`skipped_links` and rendered with them in text, markdown, and JSON (`coverage_json`, and the
`check --format json` payload). Those two fields exist for the identical reason — they record what
shaped a denominator — and the established rule is that the number is never printed without them.
A degraded run reports FEWER modules without specs than the tree holds, which is a report improved
because part of the measurement stopped.

The notice deliberately does **not** gate. `--strict` gates on `skipped_links` because a shrunken
denominator inflates a percentage that a gate then passes. A manifest notice cannot inflate the
percentage — the denominator is the stated list either way — and gating on it would put the
reported project back exactly where it started, blocked from ever gating CI.

Stderr was rejected as the channel: #570 is the standing lesson that a CI job capturing stdout sees
a clean pass while the warning goes somewhere nobody reads.

## R2 — argument, not token

In `reject_non_leading_gradle_includes`, `includeBuild` splits out of the catch-all
`token.starts_with("include")` arm and into `gradle_include_build_target`, which reuses the parser's
existing literal-only machinery rather than adding a second dialect:

- `gradle_parenthesized` for the call form (it spans lines, so only the tail of the CLOSING line is
  required to be a complete remainder — the lines after it are other directives),
  `strip_gradle_statement_terminator` for the bare Groovy form.
- `gradle_string_arguments` / `gradle_string_literal`, which already reject interpolation, dynamic
  expressions, and incomplete literals.
- `normalize_gradle_project_relative_path`, the same confinement `include(...)` paths go through, so
  `../`, absolute, drive-qualified, and UNC arguments are refused by the rule that already governs
  every other Gradle path here.

Exactly one argument is required. An accepted path is then discarded: `is_gradle_include_start`
already does not match `includeBuild`, so the module loop skips those lines untouched and `:app`
survives beside the composite build.

`includeFlat` and `includeWorkspace` stay in the token arm on purpose, and the reason is recorded in
both the code and the test: `includeFlat` resolves against the parent of the root, so its argument
is outside the project by construction, and `includeWorkspace` is not a form this parser models.
Reading their arguments would not make either supportable.

## Why the existing guard could not have caught this

`gradle_settings_reject_unsupported_include_prefixed_workspace_mutators` used `"../outside"` for
every fixture, `includeBuild` included. The guard was written against escape-the-repo paths, and
because the decision was made on the token it also caught every composite build — but no fixture
could ever tell the two apart, so the test passed identically whether the parser was reading the
path or ignoring it. A test that cannot fail for the right reason is what let this ship in rc.1 and
survive to rc.7.
