# Plan

## Delete the gate rather than restore the producer

Restoring `post-merge-archive.yml` would reinstate what #499 deliberately removed, including
the `deltas/`-as-second-archive-root bug named in its own commit message — a bug in the same
subject this release just fixed on the product side (#536).

So: remove the wait (`:198-245`) and the embedded validation (`:246-628`) from `validate`,
keeping the three checks that do not depend on the binding:

- tag version equals the `Cargo.toml` package version
- the checkout is the resolved release candidate
- the candidate is an ancestor of `origin/main`

The step is renamed from "Validate candidate source and merged archive binding" to "Validate
candidate source", and `GH_TOKEN`, `REPOSITORY` and `SERVER_URL` drop out of its `env` because
only the deleted half used them.

## State plainly what is lost

Real properties go with the block: that the bound pull request was merged, that the archive is
`workflow_version=2`, that its finalization evidence validates, that the PR has an exact
implementation head. They are unreachable today, but they are not nothing. After this change
the archive-to-merge-commit binding is enforced solely by SpecSync itself, in `change ship`
and archive validation — which is where #499 put the lifecycle. The release lane stops
re-deriving it from Git topology.

## Make the lane runnable without a tag

Add a `dry_run` boolean dispatch input. When true, `resolve` reports `mode=dry-run`; `validate`
runs because it has no guard, and every other job is already gated on `qualify` or `promote`,
so all of them skip and no tag is created.

An unrecognized value is **rejected**, not defaulted. Falling back to "not a dry run" would
turn a typo into a real promotion — and an unrecognized input silently reading as a safe
default is the exact defect class this release is about.
