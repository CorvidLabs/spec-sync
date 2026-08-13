---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: testing
---

# Testing

## Strategy

This change *removes* a failure, which is the easiest kind to get wrong: any implementation
that stops aborting looks like it works. So the assertions are built around the two ways it
could be wrong while appearing right — the exclusion going unreported, and the escape guard
weakening — rather than around the symptom that was filed.

The Rust suite proves nothing weakened. The sandbox drill proves the new behaviour is real
and, critically, that the assertions can tell a fixed binary from a broken one.

## The escape guard is the load-bearing result

48 symlink tests pass unchanged, including `safe_project_paths_reject_symlink_escapes` and
the retained-capability tests in `manifest.rs`. **If any of those had moved, the correct
response was to revert this change, not to adjust the test** — a walk that follows links is
the escape the guard exists to prevent, and that is not a trade worth making for a
usability fix.

Hand-verified with a symlinked directory pointing outside the project root: the run
completes, the link is disclosed, and the outside file's content never appears anywhere in
the output.

## Sandbox drill 040 — seven assertions

| Assertion | Guards against |
|---|---|
| baseline (no symlink) is clean | a noisy fixture masking every result below it |
| a symlink no longer aborts the run | the filed defect |
| the link is disclosed by name, **on a run that completed** | silent exclusion |
| `--strict` gates specifically on the exclusion, naming it | a partially-measured tree called clean |
| JSON carries `skipped_links` | machine consumers act on `passed`, not prose |
| an escaping link is still not traversed | the fix becoming a security hole |
| the escaping link is a disclosed skip, not fatal | inconsistent handling |

### Discrimination, measured rather than assumed

| binary | result |
|---|---|
| pre-fix | `pass=10 fail=11` |
| this change | `pass=21 fail=0` |

The two assertions that pass on **both** are the ones that should: the fixture guard, and
*"a link escaping the project root is still not traversed"* — the invariant that must hold
either way.

**Two of these assertions were wrong on the first attempt** and passed against the pre-fix
binary for entirely the wrong reason: the abort message happens to contain the link's path,
so `grep src/alias.py` matched it, and the abort exits non-zero, so a bare `exit != 0`
satisfied the `--strict` assertion. Both now require evidence the walk reached a summary.
A drill that passes on a broken binary is worse than none, because it converts "untested"
into "believed tested".

## Results

- `cargo fmt --all -- --check` clean
- `cargo clippy -- -D warnings` exit 0
- `cargo test` — **2210 unit, 331 integration, 0 failures**
- Drill 040 — **21/21**

## Gap worth naming

No test covers the *markdown* disclosure; it was verified by reading the renderer and by
the JSON and text equivalents. The renderer has no existing test harness in this change's
scope and adding one is out of scope here.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-validator-012 | `cargo test`; 48 symlink tests unchanged including the escape guard. Hand-verified: a link inside the root and a link escaping it both complete the run without traversal; the configured `source_dirs` and spec-tree sites still abort, which the surrounding tests continue to pin |
| REQ-types-006 | `cargo test`; `CoverageReport::skipped_links` is populated from a `BTreeSet`, so ordering is deterministic, and the inconclusive fallback reports an empty list rather than omitting the field — the compiler enforced the latter across eleven construction sites |
| REQ-output-003 | Drill 040 asserts the text disclosure appears on a run that reached a coverage line; the five-entry limit and remainder summary follow the same shape as the existing directory-mapping fix message |
| REQ-cmd-check-006 | Drill 040 asserts the JSON payload contains `skipped_links` naming the link; hand-verified `passed: true` alongside a non-empty list, which is the case a dashboard must be able to see |
| REQ-commands-007 | Drill 040 asserts `--strict` exits non-zero **and** names the exclusion, so it cannot pass by exiting non-zero for another reason; bare `check` asserted exit 0 in the same fixture. Both exit paths were changed after the text path was found still passing |
