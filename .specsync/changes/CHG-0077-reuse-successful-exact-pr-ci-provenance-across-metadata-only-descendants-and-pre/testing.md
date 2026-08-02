---
change: CHG-0077-reuse-successful-exact-pr-ci-provenance-across-metadata-only-descendants-and-pre
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-008` | Ancestor-reuse and trusted-policy fixtures plus hosted archive-only sandbox dogfood. |

## Characterization

- Immediate-parent lookup fails when a green product tip is followed by an unpushed review child.
- Latest-result selection rejects an exact SHA when a later cancellation follows a successful check.

## Focused regressions

- Select the nearest successful check on bounded first-parent ancestry.
- Reject second-parent, non-ancestor, another-PR, another-repository, wrong-workflow, wrong-App,
  malformed, unsuccessful-only, and over-limit evidence.
- Prefer a successful trusted-policy check for the exact SHA over newer cancelled or failed checks.
- Preserve failure when no authenticated success exists.
- Confirm review/archive metadata-only classification and product-matrix skipping remain exact.
- Traverse a prior workflow-v2 archive only when its state and finalization bind the exact parent
  commit/tree; reject a mismatched binding.
- Reject reusable job checks with run-only URLs, wrong job names, or another check-run identity.
- Preserve canonical rewritten trusted-policy URLs while ignoring later failed/cancelled runs and
  rejecting multiple successful matching runs as ambiguous.
- Preserve a successful publication when a later attempt of the same workflow run fails or is
  cancelled, and reject mismatched or unavailable run-attempt identity.
- Reject altered, omitted, extra, self-consistently forged, or self-reviewed historical archive
  evidence; authenticate sequence-ledger history and keep Git payload reads bounded to 64 MiB.
- Accept the CHG-0074 through CHG-0077 workflow-v2 archive shapes already present in repository
  history, including review evidence added during finalization and separately committed review.
- Authenticate an older valid successful policy publication when a newer successful publication is
  malformed, and fail fast above eight candidates.
- Accept an exact covered-directory `non_file` manifest entry while keeping directory objects out of
  ordinary affected-file and archive-copy discovery; require every tracked descendant and reject an
  existing directory self-consistently re-signed as `missing`.
- Mirror volatile-input filtering for broad `.specsync` scopes while retaining
  `.specsync/archive/legacy-baseline.json` as the explicit compatibility exception.
- Reconstruct valid sequence evidence after 257 updates under the committed 1000-entry bound; reject
  one-entry overflow and missing, boolean, zero, or greater-than-1000 configured limits.
- Disable Git auto-GC and auto-maintenance in the 257-update temporary repository so the stress
  fixture cannot race a background repack on hosted Ubuntu runners.

## Completion

- Run the two focused Python suites and lifecycle-workflow assertions while iterating.
- Run one final repository verification after review.
- In `CorvidLabs/spec-sync-sandbox`, push product plus metadata descendants without waiting and
  require Trust/archive reuse to bind the green product ancestor.
