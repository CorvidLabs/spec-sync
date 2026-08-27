---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: testing
---

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
