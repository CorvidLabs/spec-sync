---
change: remove-the-release-github-app-from-promotion-create-the-final-tag-with-the-workflow-s-own-github-token-and-state-who
artifact: plan
---

# Plan

Delivery scope is frozen at eight files, decided before `change new`:

| # | Path | Why it is in scope |
|---|------|--------------------|
| 1 | `.github/workflows/release.yml` | The App step, the App expressions, the `environment:` reference, and the job permission all live here |
| 2 | `.github/scripts/validate-release-candidate.py` | `UNENFORCED_TAG_POLICIES` is the enforced disclosure |
| 3 | `.github/scripts/test-validate-release-candidate.py` | Pins the promote contract; today it asserts the App must be present |
| 4 | `docs/ci-confidence.md` | "Tag authority: what is enforced, and what is not" |
| 5 | `specs/github/github.spec.md` | Invariants + Error Cases |
| 6 | `specs/github/requirements.md` | `REQ-github-007` acceptance criteria |
| 7 | `specs/github/context.md` | The narrative a later session reads first |
| 8 | `specs/github/tasks.md` | Closes the open "decide the fate of App-only final-tag creation" task |

## Order

1. **Confirm the premise before editing anything.** Live repository: variables, secrets,
   environments, webhooks, both ruleset payloads, and every workflow trigger. If any workflow,
   action, or integration depended on a final-tag push event, stop and report instead of
   proceeding — the tradeoff would be different.
2. **`release.yml`** — delete the App token step; `permissions: contents: write` on `promote` only;
   delete `environment:`; `RELEASE_TOKEN: ${{ github.token }}`; rewrite the job comment to state who
   can now mint a tag and why no environment is named. Keep the checkout, credential helper, and
   idempotent tag sequence byte-for-byte otherwise.
3. **`validate-release-candidate.py`** — add two `UNENFORCED_TAG_POLICIES` entries (token identity,
   no environment gate) and rewrite the block comment above them.
4. **`test-validate-release-candidate.py`** — replace
   `test_promotion_uses_only_the_protected_release_app` with a test that asserts the new shape and
   the absence of every App and environment reference; add a test that the disclosure comment is
   present; extend the retired-strings list to cover the whole file; update the `unenforced` payload
   assertions from two entries to three.
5. **Docs, then specs.** Prose last, so it describes what was actually built.
6. **Delta** (`deltas/github.md`) — regenerate nothing; write it by hand against the final spec text
   and check block-by-block that every `### SPEC SECTION` / `### REQUIREMENT` heading survives.
7. **Verify**: `actionlint`; the validator suite; `rulesets` against the live payloads; a local
   replay of the `unenforced` warning loop with three entries; `cargo test`; `change check`;
   `change audit --strict`.

## Not in scope

- Creating the App, the `release` environment, or a `SpecSync final tag creation` ruleset.
- Any change to the two immutability rulesets, or to `resolve`'s ruleset block beyond what the
  previous change already landed.
- Opening a pull request, merging, or finalizing the change. Commit and push a branch only.
