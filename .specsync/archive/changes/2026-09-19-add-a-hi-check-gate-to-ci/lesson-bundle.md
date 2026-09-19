# Lesson bundle — add-a-hi-check-gate-to-ci

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Add a hi check gate to CI
- **Kind**: Operations
- **Specs**: github
- **Paths**: .github/workflows/ci.yml, Cargo.lock
- **Acceptance**: A full CI run executes hi check over hi/*.md and fails the job when a file is structurally broken (duplicate id, case with no parent, id colliding with a retired one, malformed id, undeclared family, stranded criterion). A pull request touching only hi/** triggers ci.yml and reaches the required gate rather than waiting forever on a status that is never reported. The hi-check job is a dependency of implementation-gate and of the corvid-pet scoped-review job, so a structural break blocks merge and cannot post a passing review. The installed binary is human-intent pinned to 0.5.0 with --locked. Cargo.lock pins rustls at >=0.23.45 so cargo audit is not red on RUSTSEC-2026-0285.

## Evidence

- Verification commit: `e10bf2a9e24e9e988e10c72a3701c5a4b578740a`
- Base commit: `64abf58182ab222d2415d7ffcffabc9b0c8ed151`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

This repository now records its product intent with `hi` (human intent): 159 criteria
across 11 families under `hi/`, plus `INTENT.md`. Those files are the layer above the
canonical specs. A criterion is a plain sentence with a permanent id, and the one thing
`hi` guarantees is that an id is permanent and never reused.

Nothing was validating them. `hi check` catches exactly six structural problems: a
duplicate id, a case with no parent, an id colliding with a retired one, an id-shaped line
that is not a valid id, a family a file never declared, and a criterion stranded outside
every section. Three of those can otherwise go unnoticed indefinitely, because a broken
`hi/*.md` still renders as perfectly ordinary markdown in review.

What a session picking this up needs to know:

- `hi check` never fails on unfinished intent, only on a structurally broken file. A
  criterion that is false today is correct `hi`: it means the code has not arrived yet.
  This gate therefore cannot go red because somebody wrote down a want that is not built.
- The pin matters. `hi` 0.5.0 changed what `hi check` reads: a file in `hi/` whose name is
  not lowercase is hi's own rather than criteria. Pinning an older version would check
  these files with rules the tool no longer has, so the install is pinned to `0.5.0` with
  `--locked` rather than floating.
- `hi-check` also belongs in the corvid-pet job's `needs` and status table. Otherwise a
  structurally broken `hi/*.md` can still receive a passing scoped-review comment while
  `implementation-gate` is the only thing that blocks merge.
- `Cargo.lock` is a meaningful path. rustls 0.23.38 is RUSTSEC-2026-0285 (upgrade to
  >=0.23.45); without that bump every full CI run fails `cargo audit` before the new gate
  can be the story.
- `.github/` is a meaningful path under `require_change_for_meaningful_files`, which is why
  this change record exists. The lifecycle gate found the uncovered path before merge.

## From the change's testing.md

# Testing

Verified before review:

- `hi check` passes locally on this branch: `159 criteria · 11 families · 11 files`,
  exit 0, under the pinned `hi` 0.5.0.
- The pin resolves: `human-intent` 0.5.0 is published on crates.io and ships a
  `Cargo.lock`, so `--locked` cannot fail for want of a lockfile.
- The workflow still parses as YAML and reports the expected job count.
- The repository's own CI meta-validators pass: `validate-workflow-runtime-pins.py` and
  `validate-release-version.py` both exit 0.
- `cargo audit` exits 0 on the bumped lockfile (rustls 0.23.45); the remaining
  `instant` unmaintained advisory is already allowed.
- The same step shape already ran green on a clean hosted runner in a sibling repository,
  so the install path is proven rather than assumed.

Verified by CI on this pull request:

- `hi-check` runs on a full run and reports success.
- `implementation-gate` and the corvid-pet scoped-review job list `hi-check` among
  their dependencies.

Negative case worth exercising once by hand, since no automated test covers it here:
introduce a duplicate id into any `hi/*.md`, confirm `hi check` exits non-zero and that
`implementation-gate` consequently fails. Revert afterwards.

## Where these lessons go

- `specs/github/context.md`
