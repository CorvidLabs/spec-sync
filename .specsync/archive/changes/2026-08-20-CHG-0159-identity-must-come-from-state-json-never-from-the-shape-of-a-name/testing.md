---
change: CHG-0159-identity-must-come-from-state-json-never-from-the-shape-of-a-name
artifact: testing
---

# Testing

| Test | Discriminates | Proves |
|---|---|---|
| `an_undated_package_stripped_of_its_lifecycle_files_is_still_refused` | yes | the live bug: `CHG-0001-foo` lacking `-CHG-` was skipped as a tombstone |
| `a_slug_named_package_stripped_of_its_lifecycle_files_is_still_refused` | yes | no identity shape decides corruption |
| `a_deltas_only_legacy_tombstone_is_still_skipped_whatever_it_is_named` | control | a real tombstone is still skipped, so this is not "refuse everything" |
| `test-classify-ci-paths.sh` slug-only case | yes | `review_required=true` for a change with no ordinal; `false` on `origin/main` |

The vacuity control is load-bearing here. The first implementation replaced the name check with
the content check instead of joining them, and the control failed on both binaries — revealing
that a dated `deltas/`-only package had gone from refused to skipped. Without it the change
would have shipped a fail-open while claiming to close two.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-081 | The two refusal tests fail against a separate checkout of `origin/main` and pass here, covering both the undated-ordinal form that is broken today and the ordinal-free form that would break later; the tombstone control passes on both binaries, so the gate did not simply become stricter. For the CI half, the new slug-only case in `test-classify-ci-paths.sh` reports `review_required=false` against `origin/main`'s classifier and `true` against this one, which is the mandatory-review fail-open reproduced and closed |
