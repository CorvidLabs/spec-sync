# Lesson bundle — document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Document shipped SpecSync 6.0.0 and set the Action default to the stable release
- **Kind**: Documentation
- **Specs**: github, cmd_change
- **Paths**: action.yml, .github/scripts/validate-release-version.py, site/src/content/docs/integrations/github-action.md, site/src/content/docs/index.md, site/src/content/docs/quickstart.md, README.md, MIGRATION.md, docs/ADOPTING.md, docs/RELEASING.md, CHANGELOG.md, SECURITY.md, specs/github/github.spec.md, specs/github/requirements.md, specs/github/context.md, specs/github/testing.md
- **Acceptance**: User-facing docs (README, site github-action/index/quickstart, ADOPTING, MIGRATION, RELEASING, SECURITY, CHANGELOG 6.0.0 heading) describe shipped SpecSync 6.0.0: crates.io and GitHub Releases serve 6.0.0, Homebrew tap is still 5.2.0, no floating v6 tag exists, check is the product and SDD is opt-in, Trust 1.2.0 defaults specsync-version to 6.0.0. action.yml omitted-input default is 6.0.0 and github-action.md inputs table matches. validate-release-version.py PUBLISHED_ACTION_DEFAULT equals the package version. REQ-github-002 current default is the stable release. python3 -S .github/scripts/validate-release-version.py exits 0. specsync check --spec github --spec cmd_change is clean. The immutable v6.0.0 Action tag still embeds default 6.0.0-rc.14; docs tell consumers to always pass version: '6.0.0' when using that tag.

## Evidence

- Verification commit: `0d1fc8dc4514e4c30998aa153b12dccecf7ec781`
- Base commit: `b9ff32310181b796cc617406ff9298c533ebeb15`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

`v6.0.0` is a stable GitHub release (2026-09-09) at `b9ff3231`, crates.io serves `specsync 6.0.0`, and Trust 1.2.0 defaults SpecSync to 6.0.0. The tree still describes the candidate window: `action.yml` defaults to `6.0.0-rc.14`, `github-action.md` says stable is pending, CHANGELOG `## [6.0.0]` is undated with "stable publication is pending", RELEASING still treats crates.io as 5.2.0 and Homebrew bump as not done, README implies all channels may still be unpublished, and the site index leads with `change new` even though check is the product.

The tagged Action at `@v6.0.0` is immutable and still embeds default `6.0.0-rc.14`. This change updates `main` so the next Action tag defaults to 6.0.0, and tells consumers to always pass `version: '6.0.0'` on the shipped `@v6.0.0` tag.

Do not retag `v6.0.0`. Do not create floating `v6`. Homebrew tap is still 5.2.0; say so.

## From the change's design.md

# Design

No new gates. Coordinated truth-up of the Action omitted-input default and user-facing docs.

- `action.yml` default and description become 6.0.0 (stable published, pin exact, Linux/macOS only).
- Validator `PUBLISHED_ACTION_DEFAULT` equals `6.0.0`.
- Site inputs table default becomes `6.0.0`, with a callout that the immutable `@v6.0.0` tag still embeds `6.0.0-rc.14` so consumers always pass `version: '6.0.0'`.
- Index/quickstart lead with `specsync check`; SDD is optional via `change adopt`.
- README states channel truth: GitHub Releases and crates.io are 6.0.0; Homebrew is still 5.2.0; no floating `v6`.
- Trust consumers: Trust 1.2.0, omit or set `specsync-version: "6.0.0"`.
- Pre-GA journals under `docs/6-0-*` stay historical.

## From the change's testing.md

# Testing

- `python3 -S .github/scripts/validate-release-version.py` exits 0.
- `specsync check --spec github --spec cmd_change` is clean under `--strict`.
- `rg '6.0.0-rc.14' action.yml site/src/content/docs/integrations/github-action.md` is empty except the callout that the tagged `@v6.0.0` Action still embeds that default.
- REQ-github-002 evidence: `action.yml` default `6.0.0` and the site inputs table match.

## Where these lessons go

- `specs/github/context.md`
- `specs/cmd_change/context.md`
