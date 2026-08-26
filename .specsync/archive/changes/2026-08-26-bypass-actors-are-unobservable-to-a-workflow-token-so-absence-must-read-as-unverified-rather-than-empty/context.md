---
change: bypass-actors-are-unobservable-to-a-workflow-token-so-absence-must-read-as-unverified-rather-than-empty
artifact: context
---

# Context

`release.yml` failed on the rc.7 tag with:

    error: final tag immutability ruleset is missing fields: bypass_actors

GitHub returns `bypass_actors` only to a caller with **admin access to repository settings**. The
workflow runs with `contents: read, actions: read, checks: read`, so the field is absent from every
payload it fetches. The validator listed it in `REQUIRED_RULESET_FIELDS` and refused.

**This gate could never have passed from CI.** Three earlier failures hid it: first no rulesets
existed at all, then `--release-app-id ""` was rejected by argparse before a single ruleset file was
read. Each fix revealed the next layer.

It was also invisible locally, because a maintainer's `gh` is authenticated as an admin and does see
the field. The only way to observe it was to read what the runner's token can see, or to watch the
lane fail again.

## The shape

Absence and emptiness are different, and reading one as the other is the defect this release has
fixed repeatedly (#672, #684, #689's first design, #704's compatibility path). The validator
enforcing that distinction elsewhere had it backwards here: it treated an unobservable field as a
malformed payload.

Absence now means UNOBSERVED. It is validated when visible, and when not, it joins the enforced
disclosure list — the workflow already fails if that list is empty, so the admission cannot be
quietly dropped.
