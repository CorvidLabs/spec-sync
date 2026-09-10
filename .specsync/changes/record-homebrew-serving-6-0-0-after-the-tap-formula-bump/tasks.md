---
change: record-homebrew-serving-6-0-0-after-the-tap-formula-bump
artifact: tasks
---

# Tasks

- [x] Verify the tap actually serves 6.0.0 before editing any prose. `Formula/spec-sync.rb` on `homebrew-tap` `main` reads `version "6.0.0"`, and `Formula/corvid-trust.rb` reads `version "1.2.0"`.
- [x] Verify the four `spec-sync` asset checksums in the formula by downloading each release artifact and hashing it, rather than trusting the published `.sha256` sidecars. All four matched.
- [x] Confirm the installed artifact self-reports `specsync 6.0.0`.
- [x] `README.md`: fold Homebrew into the list of channels serving 6.0.0.
- [x] `MIGRATION.md`: correct the one-line Homebrew statement.
- [x] `docs/ADOPTING.md`: drop the do-not-use-the-tap instruction, keep the Linux and macOS only note.
- [x] `docs/RELEASING.md`: correct the status line and section 5, and record the `corvid-trust` test-block coupling for the next release.
- [x] `docs/ci-confidence.md`: reword the present-tense claim while keeping the lesson about not inferring channel availability from a source tag.
- [x] `site/src/content/docs/quickstart.md`: remove the inline caveat from the `brew install` line.
- [x] Grep the repository for any remaining present-tense claim that the tap serves 5.2.0. Only the reworded `ci-confidence.md` sentence matches, and it now describes the lag in the past tense.
- [x] Leave `CHANGELOG.md`, `docs/6-0-*`, `docs/GOAL-*`, `docs/superpowers/**` and `specs/**` untouched as historical records.

## Out of scope for this change

`CorvidLabs/site` carries the same caveat in `src/pages/spec-sync/docs/_content/quickstart.md` and in the 6.0.0 blog post. Those files live in another repository and are handled by a separate pull request there, so they are not tasks of this change.
