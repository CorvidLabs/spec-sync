# SpecSync 6.0.0 — independent verification of PR #774 at `444f7b91`

Verifier: Claude (verifier 3), independent of the session that wrote PR #774. Every result below was
reproduced against a binary built from the tip named, not inherited from another agent's report.

No PR was merged, no tag created, `release.yml` was never dispatched, nothing was published to
crates.io or Homebrew, no issue was closed, and `main` was never force-pushed.

## What was verified

| Item | Value |
|---|---|
| Tip verified | `444f7b916b6711bf52ab27d35d47c4afbd9578c5` (PR #774, `grok/specsync-6-remaining-p1s`) |
| Base | `main` at `7df304a2` = #773 on #772 on #771 |
| Binary | built from that tip; `specsync --version` → `specsync 6.0.0` |
| 5.2.0 binary | published `specsync-macos-aarch64` release asset; `--version` → `specsync 5.2.0` |
| Verdict | **ship at 89/90 = 98.9%**, zero first-user P1s remaining |

## Result

| Claim | Verdict | Proof |
|---|---|---|
| Defect 1 — first-user first commit | **PASS** | `init` → `add-spec greeter` → `check` → `hooks install` → `git commit` exits 0 with `git rev-list --count HEAD` = 1. The generated hook contains zero occurrences of `check --strict`, invokes plain `specsync ... check`, and its comment names `.specsync/config.toml`. Inverse holds: a phantom-export spec error blocks the commit (exit 1, HEAD 0). |
| Defect 2 — malformed `config.toml` fail-open | **PASS** | With a malformed `.specsync/config.toml`, all ten config-reading verbs exit 1, including the five that were fail-open on rc.17: `rules`, `rehash`, `compact`, `archive-tasks`, `deps`. Malformed JSON, an empty file, a directory-as-config and an invalid `enforcement` enum all fail closed. A project with no config still exits 0. |
| Defect 3 — `merge.rs` git spawn | **PASS** | `merge::unmerged_paths` now routes through `git_utils::git_cmd`; `merge::tests::unmerged_paths_uses_sanitized_git_cmd` passes on this tip. |
| C14 — 5.x upgrade end to end | **PASS** | Genuine 5.2.0 fixture with a real in-flight workflow-v1 change (5.2.0's own `change verify` executed the configured marker command) closed verify → accept → merge → archive to `No active SDD changes.`; `change adopt` wrote `cutoff_commit 610f2edb…`, equal to `git merge-base HEAD origin/main` and to the merge commit carrying the archived v1 record; a workflow-v2 change (slug id, `workflow_version: 2`) reached `.specsync/archive/changes/` with `change audit` passing and `check --strict --require-coverage 100` at 100%. |
| C23 — `MIGRATION.md` executed literally | **PASS** | `grep -n 'specsync change \(verify\|accept\|archive\)' MIGRATION.md` returns lines 31, 32 and 34 as copy-pasteable commands inside step 4, not merely a `change list` next-hint. |
| `fledge lanes run verify` | **PASS** | 5 steps in 11m 48s on this tip. |
| Required CI | **green** | 19 pass, 2 skipping on `444f7b91`, including `Required CI gate`, `trust`, `CodeQL`, `coverage`, `spec-check`. |

**Score: 89/90 = 98.9%.** Inherited 83/90 from the prior confidence report; C14 and C23 each move
fail → pass at weight 3. C34 (#532, two-clone slug collision) remains the single deferred P2 fail.

## The one commit this verification added to #774

`444f7b91` corrects `MIGRATION.md` step 4. It previously asserted that `change status` on a
workflow-v1 `Verifying` record "does not name the 6.0 `check` / `review` / `finalize` verbs".
Verified against the binary on a genuine 5.2.0-created v1 record in state `verifying`:

- the `Handoff:` line **does** name verify → accept → archive, correctly;
- the `Next:` line **does** name `change check --commit`, `change ship-status`, `change ship` and
  "finalize".

So the guide asserted a fix that had not landed. The commit points the reader at the line that is
correct rather than claiming the other is absent. Documentation only; `MIGRATION.md` is not in
`meaningful_paths`, so no change package was required. Pre-push passed before the push.

## Non-gating findings, recorded for the release decision

None of these is a first-user P1 — none is reachable by a new user on a fresh `specsync init` — so
none withholds the tag under the agreed criterion. All three are real and should be triaged for
6.0.x or handled by the operator at tag time.

1. **F2/F3 is not fixed in the binary.** The v1 `Next:` line still emits workflow-v2 verbs, as
   quoted above. Only a project holding a workflow-v1 record can reach it. The smallest correct fix
   is to guard the `Verifying` arm of the next-action on `workflow_version < 2` and emit
   verify → accept → archive, mirroring the guard the `Accepted` arm already has. That narrows
   guidance rather than widening behaviour.

2. **Two production git spawns remain unsanitized**, in the same class as defect 3 but off the
   default path: `comment::detect_branch` and `github::detect_repo` use raw `Command::new("git")`
   and run only under `check --format github`. That is the mode the GitHub Action uses in CI, where
   `GITHUB_TOKEN` is present in the environment. Routing them through `git_utils::git_cmd` would
   close the class.

3. **`action.yml` defaults to `6.0.0-rc.14`** (from #773), and
   `.github/scripts/validate-release-version.py` was changed in the same PR to *expect* that value
   when the crate version is `6.0.0`, via `PUBLISHED_ACTION_DEFAULT`. The guard that would have
   caught the mismatch now enforces it. `docs/RELEASING.md` contains no step to update
   `PUBLISHED_ACTION_DEFAULT` at tag time, so as things stand a consumer pinning
   `uses: CorvidLabs/spec-sync@v6.0.0` **without** an explicit `version:` would download release
   candidate 14 rather than the stable release they pinned. The four known consumer repositories
   (corvid-account, podo-web, podo-android, raven) all pass an explicit `version:` and are therefore
   unaffected, but the default that ships to everyone else is wrong the moment `v6.0.0` exists.
   Recommended: set the default to `6.0.0` in the release commit, or add an explicit runbook step,
   before the tag.

## Method note

One correction against my own work: early in the run a shell helper executed `cd` inside a command
substitution, so several probe commands ran in the repository checkout instead of a scratch fixture
and overwrote `.specsync/config.toml`. It was caught immediately, restored with
`git checkout -- .specsync/config.toml`, and the working tree confirmed otherwise clean. Every
defect-2 measurement reported above was re-run afterwards in isolated fixtures.

## What happens next

This document records verification only; it changes no behaviour. The sequence after review is:
squash-merge #774, cut a **new** `v6.0.0-rc.N` at the merge SHA, qualify it, dry-run the release
lane, and only then promote. `v6.0.0-rc.17` predates #774 and must not be promoted once #774 lands.
