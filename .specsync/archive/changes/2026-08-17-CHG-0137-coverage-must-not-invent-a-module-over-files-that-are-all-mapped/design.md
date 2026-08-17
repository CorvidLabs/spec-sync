---
change: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
artifact: design
---

# Design

## One rule, applied at all four sites

A candidate is reported only when it shows a gap **in its own files**:

    ModuleFileOwnership::is_uncovered()  =  owned == 0 || unmapped > 0

fed by a single-pass index (`CoverageModuleOwnership`) keyed only on the directories and stems a
candidate can actually name. One rule in one place, consulted by all four derivations, rather
than four blocks each deciding for themselves — which is how they diverged in the first place.

## The vacuity control lives in the code

`owned == 0` is not a convenience: it is the guard against over-correcting. A candidate owning no
discovered file has produced no evidence either way, and suppressing it would convert "I could not
measure this" into "this is covered" — the exact substitution this release is closing. So an
unmeasurable module keeps its report, and a test pins that.

Without the disjunct, the obvious "fix" is to delete the feature, and five tests would still pass.

## Manifest modules needed a real change, not a filter

The manifest derivation had no file information to consult. `ManifestModule::source_paths`
already existed but was dead (`#[allow(dead_code)]`), so the fix wires it rather than inventing a
parallel lookup — the field was one attribute away from being the answer.

That is the site that produced the `specsync` phantom on this repository, from `Cargo.toml`'s
package name, and it is not the site the bug report named.

## Deliberately unchanged

- **Coverage arithmetic.** The fix removes candidates from a list; it does not touch the
  numerator or denominator. `106/106` before, `106/106` after — the phantom disappears without
  moving the measurement, which is what distinguishes this from a change that quietly widened
  what counts as covered.
- **Excluded files still name modules.** Under this rule such a module has `owned == 0` and stays
  reported. Whether exclusion should suppress module derivation entirely is a separate behaviour
  decision, filed as #613 rather than folded in here.
- **`generate`'s unchecked path.** Removing the phantom route into `generate` does not fix the
  double-mapping reachable by naming a module explicitly; filed as #611.
- **Module ordering.** Two of the four blocks iterate `HashMap`s, so the reported order is
  nondeterministic. Sorting changes output shape and is filed as #612.
