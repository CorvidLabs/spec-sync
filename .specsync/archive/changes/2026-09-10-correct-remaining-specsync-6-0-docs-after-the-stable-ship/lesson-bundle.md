# Lesson bundle — correct-remaining-specsync-6-0-docs-after-the-stable-ship

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Correct remaining SpecSync 6.0 docs after the stable ship
- **Kind**: Documentation
- **Specs**: github
- **Paths**: site/, docs/, specs/github/
- **Acceptance**: docs/RELEASING.md records that promote executed for v6.0.0 on 2026-09-09 and uses a next-release example instead of a second 6.0.0 promotion
- **Acceptance**: docs/ci-confidence.md records that promote executed and does not claim the final tag is missing
- **Acceptance**: site polyglot example does not teach specsync check --explain for language detection or invented language_overrides
- **Acceptance**: site CI-gate example dual-pins CorvidLabs/spec-sync@v6.0.0 with version 6.0.0 and uses real Action inputs
- **Acceptance**: github spec companions do not require Windows for 6.0 RC qualification

## Evidence

- Verification commit: `3d3a9cc1e9de700efd3f9f3d908b8e540e5c54da`
- Base commit: `6f11e34ef65710dd8839bfaaa6e6bc9004d6a530`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

`promote` executed for `v6.0.0` on 2026-09-09 (run 34418860417) at `b9ff3231`, qualified as `v6.0.0-rc.18`. crates.io serves 6.0.0; GitHub Latest is `v6.0.0`; Homebrew still serves 5.2.0. The immutable `@v6.0.0` Action tag still embeds default `6.0.0-rc.14`.

Operator and first-user copy still described the pre-ship state: cut `rc.17` and promote, `promote` never executed, and the final tag missing. Site examples still invented `specsync check --explain` for language detection, `[[language_overrides]]`, and Action inputs `score_threshold` / `annotations`. GitHub spec companions still required Windows for RC qualification even though `REQUIRED_PLATFORMS` is Ubuntu and macOS.

Constraints: no tags, no retag of `v6.0.0`, no floating `v6`, ASCII hyphens in site copy, dual-pin `uses: CorvidLabs/spec-sync@v6.0.0` with `version: '6.0.0'`, do not claim Homebrew is 6.0.0.

## From the change's testing.md

# Testing

- `specsync check --strict github --force` passes.
- `.github/scripts/validate-release-candidate.py` `REQUIRED_PLATFORMS = ("ubuntu", "macos")`.
- `.github/workflows/release.yml` qualify matrix is `ubuntu-latest` and `macos-14` only.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-github-007 | Canonical requirement, github.spec.md promote scenario, testing.md RC rows, and context.md now bind Ubuntu and macOS only. `REQUIRED_PLATFORMS` is `("ubuntu", "macos")`. Windows is refused by the Action and is not a qualify target. |

## Where these lessons go

- `specs/github/context.md`
