---
id: a-release-candidate-must-be-installable-without-release-grade-provenance
state: archived
type: bug_fix
base_commit: 77461a4a75895593e741f73160ea62838cffd87c
---

# A release candidate must be installable without release-grade provenance

## Intent

a release candidate must be installable without release-grade provenance

## Affected Canonical Specs

- None

## Acceptance Criteria

- v6.0.0-rc.1 shipped with zero assets because the gated release lane refuses before reaching build, so every consumer of the packaged action still 404s. An audit of the six lane jobs that had never executed found four defects that are mine to fix: fetch-tags is silently ignored when fetch-depth is zero and actions/checkout then force-overwrites the annotated ref with a lightweight one, so resolve refuses every annotated tag; shell python is a literal PATH lookup with no python3 fallback and macOS runners have no bare python, which fails silently under if always and presents three jobs later as an unqualifiable candidate; the RC gate derives a workflow run id from details_url, which GitHub rewrites, a rule this repository already established in CHG-0076 and then reintroduced; and validate runs the only bare cargo invocation in any workflow, downloading a toolchain named by the candidate under test. Three further blockers are repository configuration the owner must provide and are out of scope here. Done when: those four are fixed; and a release candidate can be made installable without them, by a workflow that builds the same targets under the same names and attaches them to an existing pre-release, refusing anything that is not one.

## No-spec Rationale

Release-lane repairs and a separate pre-release asset workflow; no module contract changes and no production source in scope
