---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: plan
---

# Plan

1. **Measure before touching.** Query the live rulesets, variables, secrets, and environments; run
   the unmodified validators over both live payloads to establish that the fix is a scope change,
   not a relaxation. (Done first, deliberately — the alternative is discovering afterwards that a
   check was loosened to make reality fit.)
2. **Validator scope.** Delete `release_app_id` from `validate_tag_ruleset` and
   `validate_final_tag_creation_ruleset` with it; every remaining ruleset forbids all bypass
   actors. Reduce `rulesets_result` to two payloads.
3. **Validator honesty.** Add `UNENFORCED_TAG_POLICIES` and emit it as `unenforced`. This lands in
   the same step as the removal, so the admission cannot be forgotten between commits.
4. **Validator CLI.** Two inputs; retire the other flags so they fail as `unrecognized arguments`.
   Delete the `environment` subcommand and its helper entirely.
5. **Workflow.** Two `resolve_ruleset` calls; capture the result JSON; fail if `unenforced` is
   empty; annotate each entry as a warning and into the step summary. Remove the `RELEASE_APP_ID`
   binding, the environments queries, and `deployments: read`. Comment `promote` with its true
   provisioning status.
6. **Tests.** Rewrite `RulesetValidationTests` for two rulesets; delete
   `ReleaseEnvironmentValidationTests`; add the green-run-declares-gaps test and the workflow
   annotation contract test; extend the bypass-rejection test to four actor types on both rulesets.
7. **Docs and spec.** `docs/ci-confidence.md` rewritten to two enforced rulesets plus an explicit
   not-enforced section; `specs/github` Invariants, Error Cases, and REQ-github-007 updated through
   the semantic delta; `specs/github/tasks.md` split into done and still-open.
8. **Prove against reality.** Run the validator on the live payloads, then replay the entire
   `resolve` ruleset block verbatim against the live GitHub API. A tag cannot be pushed, so this is
   the closest achievable evidence — and it is stronger than fixture-only proof, because it
   exercises the `gh api` queries and `jq` filters too.

## Ordering constraint

Step 1 gates everything. If the live payloads had failed the unmodified strict validators, the
correct change would have been different — fix the rulesets, not the validator — and the
"no broadening" requirement would have been at risk.
