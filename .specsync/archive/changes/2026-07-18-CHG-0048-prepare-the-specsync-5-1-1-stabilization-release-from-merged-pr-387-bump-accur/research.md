---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: research
---

# Research

## Verified baseline

- GitHub Release `v5.1.0` and crates.io 5.1.0 are public.
- Main is `4652ca1`, the squash integration of fully green PR #387.
- Strict validation on that main tree passes 62/62 specs, 105/105 files, and 84,885/84,885
  covered lines; all five active changes report exact terminal evidence before archival.
- The release workflow validates that a tag matches Cargo and belongs to main, then publishes five
  archives and checksum files. It does not publish crates.io.
- `fledge release 5.1.1 --dry-run --json` is the canonical local release-plan probe; final
  publication remains deliberately separated from candidate preparation.

## Distribution gaps

- `action.yml` defaults to 5.0.0 while the latest public release is 5.1.0.
- `.github/workflows/ci.yml` exercises the packaged candidate through a runner-local mirror but
  labels that candidate as 5.0.0.
- `.github/workflows/trust.yml` labels its runner-local candidate as 5.1.0.
- README and Action documentation expose the stale default/pin.
- No `refs/heads/v5` or `refs/tags/v5` exists even though documentation recommends `@v5`.
- `CorvidLabs/homebrew-tap/Formula/spec-sync.rb` remains at 5.0.1 and has no open update PR.
- Trust PR #13 is a separate dependent rollout and currently has failing Trust gates; it is not a
  reason to weaken the SpecSync release gate.
- The post-merge Pages run and the site/extension CI jobs failed before repository commands because
  unversioned `setup-bun` attempted `https://api.github.com/repos/oven-sh/bun/git/refs/tags` during
  a GitHub API 503. Local Bun is 1.3.14, so an exact 1.3.14 workflow pin removes tag discovery while
  preserving the runtime already used for local verification.

## Decision

Use 5.1.1 as an immutable patch release. Verify the exact tag and pinned Action before creating the
floating `v5` ref. Publish Homebrew and resume the Trust rollout only after exact-version assets are
available. Never mutate `v5.1.0` or reuse the 5.1.1 version for different content.
