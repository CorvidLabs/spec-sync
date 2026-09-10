---
id: document-shipped-specsync-6-0-0-and-set-the-action-default-to-the-stable-release
state: archived
type: documentation
base_commit: b9ff32310181b796cc617406ff9298c533ebeb15
---

# Document shipped SpecSync 6.0.0 and set the Action default to the stable release

## Intent

Document shipped SpecSync 6.0.0 and set the Action default to the stable release

## Affected Canonical Specs

- `github`
- `cmd_change`

## Acceptance Criteria

- User-facing docs (README, site github-action/index/quickstart, ADOPTING, MIGRATION, RELEASING, SECURITY, CHANGELOG 6.0.0 heading) describe shipped SpecSync 6.0.0: crates.io and GitHub Releases serve 6.0.0, Homebrew tap is still 5.2.0, no floating v6 tag exists, check is the product and SDD is opt-in, Trust 1.2.0 defaults specsync-version to 6.0.0. action.yml omitted-input default is 6.0.0 and github-action.md inputs table matches. validate-release-version.py PUBLISHED_ACTION_DEFAULT equals the package version. REQ-github-002 current default is the stable release. python3 -S .github/scripts/validate-release-version.py exits 0. specsync check --spec github --spec cmd_change is clean. The immutable v6.0.0 Action tag still embeds default 6.0.0-rc.14; docs tell consumers to always pass version: '6.0.0' when using that tag.

## No-spec Rationale

Not applicable
