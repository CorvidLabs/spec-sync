# Lesson bundle — release-specsync-6-0-1-with-the-git-lfs-object-id-evidence-fix

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Release SpecSync 6.0.1 with the Git LFS object-id evidence fix
- **Kind**: Operations
- **Specs**: github
- **Paths**: Cargo.toml, Cargo.lock, action.yml, .github/workflows/ci.yml, .github/workflows/trust.yml, README.md, CHANGELOG.md, SECURITY.md, MIGRATION.md, docs/ADOPTING.md, site/src/content/docs/index.md, site/src/content/docs/quickstart.md, site/src/content/docs/integrations/github-action.md, site/src/content/examples/ci-gate.mdx, site/src/content/examples/polyglot.mdx, specs/github/github.spec.md, specs/github/requirements.md, specs/github/context.md, specs/github/testing.md, specs/github/tasks.md
- **Acceptance**: Cargo.toml and Cargo.lock are 6.0.1, the release-version validator passes, CHANGELOG has a 6.0.1 section for the Git LFS object-id evidence fix, and the Action omitted-input default on this commit is 6.0.1

## Evidence

- Verification commit: `7864698fa17f5c4daff308db3e3491e7e1600487`
- Base commit: `495f2536303673387d95b7844f71097508cea0e4`
- Verified by: `specsync check --spec cmd_change --spec github`

## From the change's context.md

# Context

`main` at `495f2536` already names a clean Git file by its object id (`specsync.project-input-digest.v4`) and accepts `filter=lfs` only for the Git LFS driver and a pointer blob. The published binary is still 6.0.0, so Peck's installed SpecSync still rejects that attribute.

This change is the 6.0.1 distribution bump on that commit. It does not merge the draft in pull request 800. The immutable `@v6.0.0` Action tag stays on its existing commit and still embeds default `6.0.0-rc.14`.

The release candidate tag is cut only after this commit is on `origin/main`. Promotion and the floating `v6` move happen after that candidate qualifies.

## From the change's design.md

# Design

6.0.1 publishes the evidence behavior already on `main`. A clean tracked file, including a clean Git LFS pointer, is named by its Git object id. Dirty and untracked bytes stay inside the existing 256 MiB bound. No evidence rule from pull request 800 is added.

The Action omitted-input default on this commit is the package version `6.0.1`. The annotated candidate is `v6.0.1-rc.1`. `release.yml` promotes that candidate to `v6.0.1`. The floating `v6` tag moves only after that, to this commit.

## From the change's testing.md

# Testing

## Requirement evidence

| ID | Evidence |
| --- | --- |
| REQ-github-002 | `python3 -S .github/scripts/validate-release-version.py` checks the Action default, consumer pin, Trust pin, README and site Action examples, the inputs-table default, and the changelog link. |

The LFS behavior this release publishes is already covered by `git_lfs_clean_file_is_named_by_its_object_id`, `clean_object_id_does_not_match_a_file_of_those_bytes`, `filter_lfs_without_the_git_lfs_driver_fails_closed`, and `filter_lfs_rejects_a_blob_that_is_not_a_pointer`.

## Where these lessons go

- `specs/github/context.md`
