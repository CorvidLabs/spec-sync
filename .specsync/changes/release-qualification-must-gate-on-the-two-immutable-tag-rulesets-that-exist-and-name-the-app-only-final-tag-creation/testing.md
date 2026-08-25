---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-007` | `RulesetValidationTests` proves the two-ruleset contract, that no bypass actor is admissible on either, and that no broadening of name/target/source/enforcement/ref-patterns/rule-types is accepted; `test_successful_rulesets_declare_every_unenforced_tag_protection` and `test_release_announces_every_unenforced_tag_protection_on_every_run` prove a green run still states both unenforced protections and that an empty statement fails the job; `test_release_queries_and_validates_exactly_the_two_immutability_rulesets` proves the App-only creation policy and the `release` environment query are gone from the workflow. Live proof: the `rulesets` command exits 0 against the actual payloads of rulesets 21432148 and 21432132, and the whole `resolve` ruleset block replays against the live GitHub API. |

## Automated

`python3 .github/scripts/test-validate-release-candidate.py` (wired through
`fledge.toml` `[tasks.release-candidate-test]`, the `verify` lane, and the RC lane). 47 tests pass.

`RulesetValidationTests` — rewritten for two rulesets:

- `test_rulesets_cli_accepts_exact_active_repository_policies` — both fixtures validate; output has
  no `final_creation` key; `--help` no longer advertises `--final-creation-ruleset-json` or
  `--release-app-id`.
- `test_successful_rulesets_declare_every_unenforced_tag_protection` — a **green** run still emits
  two `unenforced` entries naming `SpecSync final tag creation`, the release GitHub App, and the
  `'release' deployment environment`; each contains `NOT`. This is the regression guard against
  the change being quietly re-weakened into a silent pass.
- `test_rulesets_cli_requires_both_inputs_and_rejects_retired_flags` — both inputs required;
  `--release-app-id` and `--final-creation-ruleset-json` now fail with `unrecognized arguments`;
  the `ruleset` and `environment` subcommands both fail with `invalid choice`.
- `test_immutability_rulesets_reject_every_bypass_actor` — extended to `Integration`, `User`,
  `RepositoryRole`, and `OrganizationAdmin` on **both** rulesets. Previously `Integration` was
  admissible on the creation policy; now no actor type is admissible anywhere.
- Broadening guards retained unchanged: wrong name/target/source_type/enforcement, broadened or
  duplicated include/exclude patterns, unknown condition fields, missing/extra/duplicate/
  parameterized rule types, unknown top-level fields, duplicate JSON keys, non-object payloads,
  oversized payloads, duplicate ruleset ids, symlinked inputs.

`WorkflowSourceContractTests`:

- `test_release_queries_and_validates_exactly_the_two_immutability_rulesets` — exactly two
  `resolve_ruleset` calls; asserts the whole workflow file no longer contains
  `resolve_ruleset "SpecSync final tag creation"`, `--final-creation-ruleset-json`,
  `--release-app-id`, `"repos/${REPOSITORY}/environments/release"`,
  `validate-release-candidate.py environment`, or the `RELEASE_APP_ID` env binding.
- `test_release_announces_every_unenforced_tag_protection_on_every_run` — asserts the result file
  capture, the `< 1` fail-closed guard and its error text, the `::warning::` line, the
  `jq -r '.unenforced[]'` loop, and two `$GITHUB_STEP_SUMMARY` writes.
- `test_promotion_uses_only_the_protected_release_app` — unchanged and still passing; the App
  remains promotion's push mechanism.

## Live verification (cannot be covered by fixtures)

A tag cannot be pushed to test this, so the validator was run against the **actual** live payloads
rather than fixtures:

```
gh api "repos/CorvidLabs/spec-sync/rulesets/21432148?includes_parents=true" > final-immutability-ruleset.json
gh api "repos/CorvidLabs/spec-sync/rulesets/21432132?includes_parents=true" > rc-ruleset.json
python3 .github/scripts/validate-release-candidate.py rulesets \
  --final-immutability-ruleset-json final-immutability-ruleset.json \
  --rc-ruleset-json rc-ruleset.json
```

Exit 0. Also run before editing the validator: both live payloads already passed
`validate_rc_tag_ruleset` and `validate_final_tag_immutability_ruleset` unmodified, proving no
strictness was relaxed to accommodate the live configuration.

The full `resolve` ruleset block was then replayed verbatim as a shell script against the live
GitHub API — `gh api .../rulesets`, the `resolve_ruleset` helper, the validator call, the
empty-`unenforced` guard, the warning loop, and the step-summary writes — exiting 0 and printing
both warnings. Only `mapfile` was shimmed, because macOS ships bash 3.2 while runners use bash 5.

## Not verified

Nothing exercises `promote`, `build`, or `release` end to end: promotion requires a release App
that does not exist. Those jobs are unreachable on the RC-push path this change fixes and were not
altered beyond a comment.
