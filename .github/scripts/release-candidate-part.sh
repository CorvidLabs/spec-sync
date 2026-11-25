#!/usr/bin/env bash
# One slice of the release-candidate tests under cargo-nextest (REQ-github-031).
#
# RC_TEST_SLICE and RC_TEST_SLICES choose the slice: K of N, with 1 <= K <= N. cargo-nextest
# deals the whole test list across every test binary round-robin, so the N slices share no test
# and together are every test. Nothing here filters, ignores or retries a test (SPEED-2.b.2).
#
# When RC_PART_COUNTS names a file, the number of tests in this slice and in the whole list is
# written there before the run, from cargo-nextest's own listing. The release validator adds
# the slices up and refuses a platform whose slices are not the whole list. A slice that fails
# still leaves its counts.
#
# Local use, one slice of four:
#   RC_TEST_SLICE=1 RC_TEST_SLICES=4 fledge lanes run release-candidate-tests
set -euo pipefail

slice="${RC_TEST_SLICE:-}"
slices="${RC_TEST_SLICES:-}"
if [[ ! "$slice" =~ ^[1-9][0-9]*$ || ! "$slices" =~ ^[1-9][0-9]*$ ]] || (( slice > slices )); then
  echo "error: RC_TEST_SLICE and RC_TEST_SLICES must be whole numbers with 1 <= slice <= slices (got '${slice}' of '${slices}')" >&2
  exit 2
fi
partition="slice:${slice}/${slices}"

if [[ -n "${RC_PART_COUNTS:-}" ]]; then
  listing="$(mktemp -d)"
  trap 'rm -rf "$listing"' EXIT
  cargo nextest list --message-format json > "${listing}/all.json"
  cargo nextest list --message-format json --partition "$partition" > "${listing}/part.json"
  python3 -I - "${listing}/all.json" "${listing}/part.json" "$RC_PART_COUNTS" <<'PY'
import json
import sys


def matched(path):
    """Count the tests a run with this listing's filter would run."""
    with open(path, encoding="utf-8") as source:
        listing = json.load(source)
    return sum(
        1
        for suite in listing["rust-suites"].values()
        for case in suite["testcases"].values()
        if case["filter-match"]["status"] == "matches"
    )


listed, in_part = matched(sys.argv[1]), matched(sys.argv[2])
if not 0 < in_part <= listed:
    sys.exit(f"error: the slice lists {in_part} tests and the whole list {listed}")
with open(sys.argv[3], "w", encoding="utf-8") as target:
    json.dump({"tests_in_part": in_part, "tests_listed": listed}, target, sort_keys=True)
    target.write("\n")
print(f"slice holds {in_part} of {listed} tests")
PY
fi

cargo nextest run --no-fail-fast --partition "$partition"
