---
change: CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2
artifact: design
---

# Design

## Change sequence integrity

Add a versioned `.specsync/change-sequence.json` ledger containing the latest sequence claim and an exact list of acknowledged pre-ledger collision groups. `change new` updates the claim under the existing project lock when it creates a workspace. Parallel branches created from one base produce different claim content, forcing reconciliation before merge. Project checking scans active and archived state, groups records by numeric sequence, and rejects every collision group not exactly represented by the historical baseline. The repository migration records the existing three-ID `0016` group without changing any accepted artifact or evidence.

## Verification execution

Configured verification children inherit a private recursion-depth marker. Any nested SpecSync lifecycle check returns one actionable cycle diagnostic before it can execute configured commands again. Direct `specsync check` commands are rejected before process creation. Each verification attempt is appended to a versioned history file while `verification.json` remains the compatible latest projection. A failed native attempt can be retried from `verifying`; unrelated failed or stale changes remain errors.

## Canonical successors

When validating an accepted stale predecessor, suppress only its delivery-input stale error when a later semantic successor has a current definition approval and exactly governs every affected module and path. An implementing successor is eligible so it can reach verification. A verifying successor is eligible only with current passed evidence. Failed, draft, abandoned, no-spec, partial-scope, or stale-definition successors never suppress the predecessor. Accepted recorded successors continue governing through immutable history.

## Canonical path resolution

Resolve every affected module through `.specsync/registry.toml` when an entry exists, validate the registered path inside the project root, and derive `requirements.md` from the registered spec's parent directory. Conventional `specs/<module>/<module>.spec.md` remains the fallback for projects without a registry entry.

## Static coverage and scaffold completeness

Coverage treats configured static extensions such as HTML as measurable files while keeping export extraction optional for formats without symbols. Known generated companion markers are matched by artifact type and exact trimmed line outside fenced code. They produce actionable warnings containing artifact path and line; strict enforcement promotes those warnings to failure.
