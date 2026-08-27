# Lesson bundle — a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token
- **Kind**: BugFix
- **Specs**: manifest, validator, types, config, output, cmd_check, comment, cli, generator
- **Paths**: src/manifest.rs, src/validator.rs, src/types.rs, src/config.rs, src/output.rs, src/commands/check.rs, src/comment.rs, src/main.rs, src/generator.rs, tests/integration/commands.rs
- **Acceptance**: a project whose Gradle settings cannot be parsed but whose source_dirs is explicitly configured runs check and coverage to completion and reports real numbers; the unparseable manifest is disclosed as a coverage notice beside those numbers, in text, markdown, and JSON, rather than replacing them; that notice never gates, because unlike a shrunken denominator it cannot inflate a percentage; degrading reads no byte outside the project root, generates nothing out of the rejected discovery, and leaves the outside tree untouched; a project that did NOT configure source_dirs still fails closed, because its source list came from the discovery that failed; an in-repo includeBuild("vendor/podo-shared") parses, contributes no module, and leaves the root build's include list untouched, wherever the declaration appears; includeBuild("../outside") and dynamic, interpolated, multi-argument, or block-suffixed includeBuild arguments are still refused, now naming the argument rather than the token

## Evidence

- Verification commit: `2fe96060393061bf5266d3a4cae08eea2dfe828d`
- Base commit: `48e9da28ac45d3bd1d3a759e6142bb3812f3d53c`
- Verified by: `cargo test validator::tests::`, `cargo test commands::check::tests::`, `cargo test cli::tests::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

## What led here

A 6.0 release blocker reported from the field (#723). A Gradle project with a valid in-repo
composite build — `includeBuild("vendor/podo-shared")` in `settings.gradle.kts` — cannot run
`check --strict` or `coverage` on **any** candidate from rc.1 through rc.7. `view` and `change new`
work. They have `source_dirs = ["app/src/main/java"]` explicitly configured and it does not rescue
them. v5.2.0 works and reports 24/29 files covered (82.76%).

That split is the worst possible one: the tool is usable for authoring and unusable for the thing CI
depends on, so a project can adopt it, get value, and only discover the gap when it tries to enforce
anything. And there is no v6 to pin — every candidate is affected, so it is not "upgrade later", it
is "6.0 cannot ship to this adopter".

Reproduced exactly before any change was made, against `target/release/specsync` built from `main`:

```
$ specsync coverage --root repro
Coverage inconclusive: Cannot parse Gradle settings manifest settings.gradle.kts:
  Unsupported Gradle workspace mutator includeBuild
COVERAGE EXIT: 1
CHECK EXIT: 1
```

## Two defects, and why the second one was fixed first

**A.** `src/manifest.rs` rejected `includeBuild` on the token prefix alone. The path was never read,
so an in-repo composite build and an escape out of the repository produced the identical hard `Err`.

**B.** `src/validator.rs` propagated a manifest-discovery failure with `?` even when the project had
explicitly configured `source_dirs`. Every other call site already degrades — `config.rs:66` uses
`unwrap_or_else`, `validator.rs:430` falls back to a scan — coverage did not.

B is the one that removes a class. Discovery exists to INFER what the user did not state; when the
user has stated it, a failure to infer must not veto it. Fixing only A would leave the next
unreadable manifest, in any ecosystem, able to override an explicit declaration all over again.

This is the same shape the release has fixed repeatedly — an input the tool could not interpret,
treated as a verdict about the project: #672 (an unparseable schema reported every table as
missing), #684 (a missing `schema_dir` gated a release), and the `bypass_actors` field a runner
cannot read. Here it is one layer out.

## Already ruled out

- **An escape hatch to disable manifest discovery.** There is none today. An opt-out is a worse
  answer than not needing one, and it would leave the class intact.
- **Skipping discovery entirely when `source_dirs` is configured.** That would silently change
  module attribution for every project that configures `source_dirs` AND has a valid manifest.
  Discovery still runs; only its FAILURE is reinterpreted.
- **Making the notice gate `--strict`.** It cannot inflate a percentage (unlike `skipped_links`,
  which shrinks the denominator), and gating on it would put the reported project back exactly where
  it started — able to run, unable to gate.
- **Reporting the degradation on stderr only.** #570 is the standing lesson: a CI job capturing
  stdout reads a clean pass while the warning goes where nobody looks.
- **Parsing the included build's own settings.** A composite build is a separate build; discovering
  its modules is a feature, not this fix. Accept-and-ignore is strictly better than aborting.

## Constraints worth knowing

- `is_gradle_include_start` already does not match `includeBuild`, so accepting one requires no
  change to the module loop — an accepted composite build is naturally skipped.
- Clippy is NOT in this project's `change check` verification commands (`fledge.toml` puts `lint` in
  the `verify`/`ci` lanes), so `change check` passes green while CI blocks the PR. Run
  `cargo clippy -- -D warnings` — bare, not `--all-targets` — by hand.

## Two things the first cut got wrong

Recorded while fresh, because both are the kind of thing the next change here will hit.

**The same conditional composite build got two different verdicts.** Judging `includeBuild` by its
argument left its POSITION unjudged, and the remainder check could not tell a trailing configuration
block (`{ dependencySubstitution … }`, which this parser does not model) from the closing brace of an
enclosing `if`. So `if (x) { includeBuild("vendor/s") }` was refused on one line and accepted across
three — only in the first case did the `}` land on the declaration's own line. A verdict that turns
on where the author pressed Enter is a bug whichever way it is settled. Settled by accepting both: a
composite build contributes no module whether or not its branch runs, so its position cannot change
what is discovered. That is deliberately ASYMMETRIC with `include`, which is still refused when
conditional — a conditional `include` does change the module set.

**Six integration tests encoded the old contract, and `change check` is what found them.** Every
`gradle_*_is_inconclusive_for_coverage_gating_commands` fixture calls `setup_minimal_project`, which
states `sourceDirs: ["src"]` — so they were all asserting the CONFIGURED case, which is exactly the
case this change stops treating as fatal. They were not wrong about safety; they were using the exit
code as a proxy for it. Each now clears `sourceDirs` so it still asserts the fail-closed contract on
an inferred source list, and a new test asserts the degraded half over the same unsafe shapes: no
outside byte read or disclosed, nothing generated from the rejected discovery, the outside tree
untouched. Asserting those directly rather than through exit status is what makes the pair stronger
than the original six.

`report` and `score` exit 1 in those fixtures for a reason that has nothing to do with the manifest —
they are not git repositories, so staleness is unmeasurable. The degradation test therefore judges
those two on the report they produced rather than on their status; only `check`, `coverage`, and
`generate` are asserted to exit 0.

## From the change's design.md

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

## From the change's testing.md

# Testing

Three tests, each of which was confirmed to FAIL against the unfixed code before it was relied on.
The existing guard here is the cautionary case: it passed for years without ever being able to fail
for the right reason, which is exactly how this shipped in rc.1 and survived to rc.7.

## The tests

- **`gradle_settings_accept_an_in_repo_include_build`** (`src/manifest.rs`) — the regression, and the
  case that had NO coverage at all. Five shapes of the same in-repo composite build (Kotlin call,
  bare Groovy, `settings.`-qualified, a path needing normalization, and a multi-line call). Each must
  parse AND leave `:app` as the only module, so accepting a composite build cannot quietly change
  what the root build declares. Unfixed: `in-repo includeBuild refused: Unsupported Gradle workspace
  mutator includeBuild`.

- **`gradle_settings_still_refuse_an_escaping_or_dynamic_include_build`** (`src/manifest.rs`) —
  **honest label: the refusals asserted first are the CONTROL, and they pass on the unfixed parser
  too. That is the point.** Nine fixtures: `../outside`, `/etc/outside`, `vendor/../../outside`, the
  `settings.`-qualified escape, `${…}` interpolation, a bare identifier, two arguments, a trailing
  configuration block, and `"."`. What breaks this loop is a relaxation that stopped confining the
  path or stopped requiring one complete literal. The DIAGNOSTICS asserted second are what is
  genuinely new — the old refusal said `Unsupported Gradle workspace mutator includeBuild` about
  every one of these and about the valid in-repo build alike, so it could not say which argument was
  wrong. That second half is why this test fails unfixed.

- **`configured_source_dirs_survive_a_manifest_that_cannot_be_parsed`** (`src/validator.rs`) — the
  precedence, and the half that would have unblocked the reporter on its own. A tree with a
  malformed `settings.gradle` and `source_dirs = ["app/src/main/java"]` in `.specsync/config.toml`,
  loaded through `retained_config` so the real config path is exercised rather than a hand-built
  struct. Asserts coverage COMPLETES, reports the real file (`total_source_files == 1`, the Java
  file named), and carries exactly one notice that names what could not be read.
  **Both halves are in one test on purpose**: the second half flips `source_dirs_set` off over the
  same tree and requires the same error to still be fatal. Asserting only the first half would pass
  equally well against a change that merely stopped failing — and this is a precedence, not a
  softening. Unfixed: `configured source_dirs were vetoed by: Cannot parse Gradle settings manifest
  settings.gradle: Gradle include declaration has unbalanced parentheses`.

  The manifest here is MALFORMED rather than merely unsupported, deliberately: it keeps the
  precedence test independent of which Gradle forms the parser happens to accept, so fixing A cannot
  make it vacuous.

## Verifying they fail unfixed

Both fixes were temporarily disabled in place (`false &&` on the two new branches) and the three
tests re-run:

```
test manifest::tests::gradle_settings_accept_an_in_repo_include_build ... FAILED
test manifest::tests::gradle_settings_still_refuse_an_escaping_or_dynamic_include_build ... FAILED
test validator::tests::configured_source_dirs_survive_a_manifest_that_cannot_be_parsed ... FAILED
test result: FAILED. 0 passed; 3 failed
```

Each failed with its own message, quoted above — not a shared compile error.

## The existing guard, corrected rather than extended

`gradle_settings_reject_unsupported_include_prefixed_workspace_mutators` used `"../outside"` for all
four of its fixtures. `includeBuild("../outside")` was removed from that list, because it no longer
belongs to the token arm: it is refused by path confinement now, and leaving it there would assert
the wrong reason. `includeFlat`, `includeWorkspace`, and the qualified `settings.includeFlat` stay,
and the comment records why reading THEIR arguments would not make them supportable.

## End-to-end, on the reported shape

The reported project shape (`includeBuild("vendor/podo-shared")`, `include(":app")`,
`source_dirs = ["app/src/main/java"]`, one Java source) run against release builds of both trees:

| | before | after |
|---|---|---|
| `coverage` | exit 1, `Coverage inconclusive: … Unsupported Gradle workspace mutator includeBuild` | exit 0, real figures |
| `check --strict` | exit 1, same message | exit 0 |

The second half of the class was checked too — the same tree with a MALFORMED `settings.gradle`
(one the parser will never accept) plus the configured `source_dirs`: `coverage` completes and
prints the `⚠` notice. That is the case that proves fixing B alone unblocks the adopter regardless
of what `includeBuild` support means.

## The six tests that encoded the old contract

`change check` failed on six existing integration tests, and they were right to fail: every
`gradle_*_is_inconclusive_for_coverage_gating_commands` fixture builds on `setup_minimal_project`,
which states `sourceDirs: ["src"]`. They were all asserting the CONFIGURED case — the one this
change stops treating as fatal — while reading as though they asserted safety.

They were not wrong about safety. They were using the exit code as a proxy for it. Both halves are
now asserted directly:

- Each of the six calls the new `omit_config_source_dirs` helper, so it asserts the fail-closed
  contract on an INFERRED source list: inconclusive, exit 1, no outside bytes disclosed, no
  mutation, outside tree untouched. Unchanged behaviour, now unambiguously the case it names.
- `unsafe_gradle_discovery_degrades_over_stated_source_dirs_without_escaping` asserts the other half
  over four unsafe shapes (malformed, root escape, interpolated `projectDir`, and an unsupported
  `includeFlat` mutator): the commands complete, `check` and `coverage` carry a `manifest_notices`
  entry naming the failure, **and every protection the refusal was providing still holds** — no byte
  outside the root disclosed, nothing generated from the rejected discovery, outside bytes unchanged.
  It fails against the unfixed code (`Unexpected failure. code=1`).

That pair is stronger than the original six, because the safety properties are now asserted as
themselves rather than inferred from a non-zero exit.

## Suite

`cargo test`: 2389 unit + 406 integration tests pass.
`cargo clippy -- -D warnings` (bare, matching the `lint` task in `fledge.toml`) is clean — it is NOT
among the commands `change check` runs, so it was run by hand.

## Where these lessons go

- `specs/manifest/context.md`
- `specs/validator/context.md`
- `specs/types/context.md`
- `specs/config/context.md`
- `specs/output/context.md`
- `specs/cmd_check/context.md`
- `specs/comment/context.md`
- `specs/cli/context.md`
- `specs/generator/context.md`
