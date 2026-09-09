# Releasing SpecSync

Operator runbook for cutting a stable `vX.Y.Z` from an immutable release candidate, derived from
`.github/workflows/release.yml`, `.github/workflows/rc-assets.yml`, and
`.github/scripts/validate-*.py` in this tree. Where a step has never executed here, it says so.
Read `docs/ci-confidence.md` ("Tag authority" and "What has actually run") first.

Shape of the lane: an annotated `vX.Y.Z-rc.N` tag push **qualifies** a candidate on Ubuntu and
macOS (`REQUIRED_PLATFORMS` in `validate-release-candidate.py`). A later `workflow_dispatch`
**promotes** that exact candidate: mints `vX.Y.Z`, builds five binaries, creates the GitHub
release. crates.io and Homebrew are manual and come after.

## 1. Preconditions

- `main` is green and the commit you intend to ship is on `origin/main`. `validate` refuses any
  candidate that is not an ancestor of `origin/main`.
- No active change workspace: `.specsync/changes/` contains no change directories (`init` creates the
  directory and finalization leaves it in place, so check its contents, not its existence:
  `specsync change list` prints `No active SDD changes.`).
- `Cargo.toml` `[package] version` and `Cargo.lock` carry the exact version the tag will name
  (`6.0.0`). `validate` parses the RC tag `vX.Y.Z-rc.N` and requires its `X.Y.Z` component to equal
  the Cargo version; a mismatch fails the run.
- The three validators pass at the candidate commit:

```bash
python3 .github/scripts/validate-release-version.py
python3 .github/scripts/validate-workflow-runtime-pins.py
python3 .github/scripts/test-validate-release-candidate.py
```

`validate-release-version.py` checks, against the Cargo version: `Cargo.lock`, the `action.yml`
`version` default, the `ci.yml` `action-consumer` step (`version:` plus the runner-local mirror
URL), the `trust.yml` pin and mirror, exact-head checkouts in `spec-check`/`trust`, every
`uses: CorvidLabs/spec-sync@vX.Y.Z` and `version:` in `README.md` and `site/src/content`, the
`| version |` default row in `site/src/content/docs/integrations/github-action.md`, exactly one
`@vX` in `SECURITY.md`, a `## [X.Y.Z]` heading in `CHANGELOG.md`, and the
`[Unreleased]: .../compare/vX.Y.Z...HEAD` link. `validate-workflow-runtime-pins.py` checks that
`oven-sh/setup-bun@v2` with `bun-version: 1.3.14` appears exactly once in `ci.yml` `site`,
`ci.yml` `vscode-extension`, and `pages.yml` `build`, and nowhere else.

- Consumer pins (#647). `action.yml` at a consumer's own `uses:` ref decides the binary: with no
  explicit `version:` input the wrapper uses that revision's embedded default, which is `latest`
  for the `@v4` / `@v4.5.0` wrappers and `6.0.0` for `@v6.0.0` and `@main`. Publishing a
  non-prerelease repoints `releases/latest`, so every consumer on an old wrapper without an
  explicit `version:` moves with it. Before promoting, confirm each known consumer passes an
  explicit `version:`:

```bash
for r in corvid-account podo-web podo-android; do
  gh api "search/code?q=repo:CorvidLabs/$r+spec-sync+path:.github/workflows" --jq '.items[].path' |
  while read -r p; do
    gh api "repos/CorvidLabs/$r/contents/$p" --jq .content | base64 -d | grep -n -A4 'spec-sync@'
  done
done
```

As of this document all three carry `version: "6.0.0-rc.12"`; `raven` pins `4.5.0`.

## 2. Cut the candidate

The tag must be **annotated**, point **directly at a commit**, and be a **new, non-forced** push;
`resolve` rejects lightweight tags, re-pushed tags, and any tag name whose earlier release runs
carry a different head SHA (that name is then refused permanently: pick the next `N`).

```bash
git fetch origin main
git tag -a v6.0.0-rc.17 <sha-on-origin/main> -m "SpecSync 6.0.0 release candidate 17"
git push origin refs/tags/v6.0.0-rc.17
```

The push matches `on.push.tags: v*.*.*-rc.*` and runs `release.yml` in `qualify` mode:
`resolve` (checks both immutable-tag rulesets, warns about the protections it does not enforce)
-> `validate` (ancestor-of-main, Cargo version equals tag version) -> `qualify` on
`ubuntu-latest` and `macos-14` (`fledge lanes run release-candidate`, i.e. `test` + `build`)
-> `record-qualification`. Watch it:

```bash
gh run list --workflow release.yml --limit 5
gh run watch <run-id>
```

"Qualified" means all of the following, and `authorize-release` later re-checks each one:

- The run has two artifacts, `rc-evidence-ubuntu` and `rc-evidence-macos`, each a JSON record with
  `outcome: success`, the RC tag, the candidate SHA, `lane: release-candidate`, and one shared
  `workflow_revision`.
- `record-qualification` posted a check run named `SpecSync release candidate` on the candidate
  SHA with `external_id` `specsync-rc:<rc_tag>:<sha>` and conclusion `success`:

```bash
gh api repos/CorvidLabs/spec-sync/commits/<sha>/check-runs \
  --jq '.check_runs[] | select(.name=="SpecSync release candidate") | .conclusion + " " + .external_id'
```

A pushed RC tag creates **no** GitHub release and no binaries. If testers need an installable
candidate, create a pre-release by hand, then dispatch `rc-assets.yml`, which refuses anything
that is not an existing published pre-release and attaches the same five archives `build`
produces. Optional; a candidate can go straight to promotion. `--verify-tag` is required: without
it, a mistyped or missing tag makes `gh release create` mint that tag from the default branch,
and the ruleset then makes the unqualified tag permanent.

```bash
gh release create v6.0.0-rc.17 --verify-tag --prerelease --title "SpecSync 6.0.0-rc.17" --notes "Release candidate"
gh workflow run rc-assets.yml --ref main -f tag=v6.0.0-rc.17
```

## 3. Dry run

`dry_run=true` runs `resolve` and `validate` only; every other job is gated on the mode and
skips. No tag is created, nothing is published. Dispatch must come from `main`; `resolve` rejects
any other `workflow_ref` and any `dry_run` value that is not exactly `true`/`false`.

```bash
gh workflow run release.yml --ref main -f rc_tag=v6.0.0-rc.17 -f dry_run=true
```

## 4. Promote

**`promote` has never executed in this repository.** Its git mechanics were rehearsed against a
local bare repo; `GITHUB_TOKEN` pushing against the live ruleset first happens on the real
release. There is no throwaway target: `final_tag` comes from the candidate's `Cargo.toml`.

```bash
gh workflow run release.yml --ref main -f rc_tag=v6.0.0-rc.17
```

What runs, in order:

1. `resolve`, `validate` as above.
2. `authorize-release`: the newest `SpecSync release candidate` check run for this exact
   `rc_tag`/SHA must be `success`; a successful `push` run of `release.yml` for that tag must
   exist; it downloads exactly two `rc-evidence-*` records from that run and runs the validator in
   `promote` mode (or `release` mode if `vX.Y.Z` already exists).
3. `promote`: with `contents: write` on this job only, creates annotated `v6.0.0` at the
   candidate SHA and pushes it with `GITHUB_TOKEN`. No release App, no protected environment,
   no second approver: anyone who can dispatch `release.yml` from `main` can mint the tag.
4. `build`: five targets, cold build, no cache: `specsync-linux-x86_64`,
   `specsync-linux-x86_64-musl`, `specsync-linux-aarch64`, `specsync-macos-x86_64`,
   `specsync-macos-aarch64`, each as `<name>.tar.gz` + `<name>.tar.gz.sha256` (+ a
   `.provenance.json` kept only as a workflow artifact).
5. `release`: revalidates artifact checksums and identity, re-runs the validator in `release`
   mode against the final tag, refuses if a GitHub release for `v6.0.0` already exists, then
   creates the release (`softprops/action-gh-release`, generated notes, `.tar.gz` and `.sha256`
   files only). It is **not** marked pre-release, so `releases/latest` moves to it.

Idempotent and refusal cases:

- `vX.Y.Z` already exists **at the candidate SHA**: `promote` logs "already identifies the
  qualified candidate; continuing idempotently" and the run proceeds to `build`/`release`.
  This is the safe path if a later job failed and you re-dispatch the same `rc_tag`.
- `vX.Y.Z` exists **at a different commit**: `promote` fails with
  `already points at a different commit`. Nothing can move that tag; ship a new version.
- A GitHub release for `vX.Y.Z` already exists: `release` fails; assets are never replaced.
- `promote` fails: the tag is pushed as the last action of the job, so a failure leaves the tag
  namespace untouched. Read the log, fix the cause on `main`, and re-dispatch. A `resolve`,
  `validate`, or `authorize-release` failure likewise creates nothing.

## 5. After the tag

Verify the release and its ten assets, and check one archive against its sidecar:

```bash
gh release view v6.0.0 --json isPrerelease,assets --jq '.isPrerelease, (.assets[].name)'
gh release download v6.0.0 -p 'specsync-macos-aarch64.tar.gz*' -D /tmp/specsync-6
(cd /tmp/specsync-6 && shasum -a 256 -c specsync-macos-aarch64.tar.gz.sha256)
```

Publish to crates.io (currently serving 5.2.0) from the tagged tree. `Cargo.toml` `include`
limits the package to `src/`, `Cargo.toml`, `Cargo.lock`, `README.md`, `CHANGELOG.md`, `LICENSE`.
Publishing is permanent; a version can only be yanked.

```bash
git checkout v6.0.0
cargo publish --dry-run --locked
cargo publish --locked
```

Bump the Homebrew tap (`github.com/CorvidLabs/homebrew-tap`, `Formula/spec-sync.rb`). The
formula downloads `specsync-<os>-<arch>.tar.gz` from
`releases/download/v#{version}/`, so the fields that change are `version "5.2.0"` -> `"6.0.0"`
and the four `sha256` values (macOS arm/intel, Linux arm/intel), taken from the `.sha256`
sidecars of `specsync-macos-aarch64`, `specsync-macos-x86_64`, `specsync-linux-aarch64`,
`specsync-linux-x86_64`. The musl archive is not used by the formula. Open a PR on the tap, then:

```bash
brew update && brew install CorvidLabs/tap/spec-sync && specsync --version
```

Date the changelog. `## [6.0.0]` has no date during the candidate window; open a PR on `main`
that changes it to `## [6.0.0] - YYYY-MM-DD`, removes the "stable publication is pending" line,
and leaves the `[Unreleased]: .../compare/v6.0.0...HEAD` link and the `[6.0.0]:` release link
in place. `validate-release-version.py` still passes because it matches the `## [6.0.0]` prefix.

Floating major tag. `release.yml` creates only `vX.Y.Z`; the floating `v6` ref that `SECURITY.md`
and the README describe (as `v1` and `v4` exist today) is created or moved by hand, and only after
the release has passed its platform smoke tests. Consumers who pin `@v6.0.0` are unaffected.

```bash
git fetch origin --tags
git tag -f v6 v6.0.0
git push --force origin refs/tags/v6
```

Announce: link the release, say `releases/latest` now serves 6.0.0, point at `MIGRATION.md`.

## 6. Rollback, and what cannot be undone

- **Tags are immutable.** Both rulesets (`SpecSync immutable RC tags`, `SpecSync immutable final
  tags`) block `update` and `deletion` with no bypass actors. A wrong `vX.Y.Z` or `-rc.N` cannot
  be moved or removed. The remedy for a bad stable release is a new version (`6.0.1`) through
  the same lane; the remedy for a bad candidate is the next `N`.
- **Release assets are not replaced.** `release` refuses an existing release and uploads with
  `overwrite_files: false`; the lane cannot be re-run to swap binaries. To move `releases/latest`
  back to the previous release without reclassifying the bad one, run
  `gh release edit <previous-tag> --latest`.
- **crates.io is append-only.** `cargo yank --version 6.0.0` hides it from new resolutions;
  the version cannot be republished.
- **Homebrew tap and CHANGELOG are ordinary git.** Revert the formula PR to move users back.
