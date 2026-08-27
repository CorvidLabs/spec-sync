---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: tasks
---

# Tasks

- [x] Reproduce the reported failure against a `main` release build before editing anything
- [x] Establish what `compute_coverage_checked` actually uses the manifest for, so degrading is a
      reasoned choice rather than a hope
- [x] R1: record whether `source_dirs` was stated (`SpecSyncConfig.source_dirs_set`) in both config
      loaders and in `retained_config`, from the same predicate that decides its own fallback
- [x] R1: `retained_coverage_manifest` — propagate when the source list came from discovery,
      degrade to a notice when it did not
- [x] R1: `CoverageReport.manifest_notices`, rendered in text, markdown, and both JSON payloads,
      and deliberately not gating
- [x] R2: `gradle_include_build_target` — judge `includeBuild` by its argument, reusing the existing
      literal-only helpers and the same path confinement `include(...)` uses
- [x] R2: leave `includeFlat`/`includeWorkspace` in the token arm, with the reason recorded
- [x] Remove `includeBuild("../outside")` from the token-arm fixture list it can no longer belong to
- [x] Add `gradle_settings_accept_an_in_repo_include_build`
- [x] Add `gradle_settings_still_refuse_an_escaping_or_dynamic_include_build` (honest label: the
      refusals are the control and pass unfixed; the diagnostics are what is new)
- [x] Add `configured_source_dirs_survive_a_manifest_that_cannot_be_parsed`, both halves
- [x] Confirm all three FAIL against the unfixed code, each for its own reason
- [x] Re-run the reproduction against the fixed binary
- [x] Correct the two `specs/manifest/requirements.md` lines that state the old contract
- [x] Bump and log manifest 19, validator 37, types 13, config 22, output 9, cmd_check 25
- [x] Fold the lesson into `specs/manifest/context.md` and `specs/validator/context.md`
- [x] `cargo fmt`, `cargo test`, `cargo clippy -- -D warnings` (bare — not in `change check`)
- [x] Resolve the one-line/multi-line disagreement on a conditional `includeBuild`
- [x] Re-point the six `gradle_*_is_inconclusive_*` integration tests at an INFERRED source list, and
      add `unsafe_gradle_discovery_degrades_over_stated_source_dirs_without_escaping` for the other
      half — asserting the safety properties directly rather than through the exit code
- [x] `change check --commit`, `change audit --strict`

## Deliberately not done

- Discovering modules or source directories from inside an accepted composite build. Separate build,
  separate manifest; that is a feature, not this fix.
- Gating `--strict` on a manifest notice. It cannot inflate a percentage, and gating on it would put
  the reported project back where it started.
- An escape hatch to disable manifest discovery. Not needing one is the better answer.
