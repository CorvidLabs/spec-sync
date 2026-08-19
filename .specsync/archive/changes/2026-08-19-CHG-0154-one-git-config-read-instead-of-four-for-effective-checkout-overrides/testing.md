---
change: CHG-0154-one-git-config-read-instead-of-four-for-effective-checkout-overrides
artifact: testing
---

# Testing

## The right bar here is equivalence, not discrimination

Every other change in this release had to *fail* on an unfixed binary. This one must not. It is
a behaviour-preserving refactor, so a test that only the new code can satisfy would be testing an
implementation detail rather than the property.

Both new tests, run against a binary built from a separate checkout at `03210d94`:

| test | unfixed | fixed |
|---|---|---|
| `one_config_read_matches_four_for_every_case_git_distinguishes` | ok | ok |
| `a_malformed_config_fails_loudly_rather_than_reading_as_unset` | ok | ok |

The second one initially **failed** on the unfixed binary — not because the property differed,
but because it asserted the new message wording. Batching changed the text from naming one key
to naming the group. The assertion now checks that a broken config errors and names the
inspection failure, which is true of both. Recorded here because the first version looked like
evidence and was not.

The three existing precedence tests (`checkout_autocrlf_resolution_honors_local_global_and_injected_values`
and its siblings) now drive the new snapshot path and pass unchanged — that is what keeps the
test-only helper from drifting into a parallel implementation.

## Equivalence, verified against git 2.50.1 before writing code

| case | expectation | confirmed |
|---|---|---|
| multi-valued key | last value wins | `--get-regexp` lists in order; last matches `--get` |
| valueless key | empty value | record carries no `\n` |
| mixed-case `[CORE] FileMode` | key lowercased | yes |
| whitespace around value | trimmed | yes |
| nothing set | rc=1, empty stdout **and** stderr | identical to `--get` |
| malformed config | rc=128 with stderr | identical to `--get` |

## Performance, measured by me on an idle machine

Sequential, release, both test binaries prebuilt so compilation is outside the timed window,
zero other processes, orphaned watchers cleared first:

```
UNFIXED   593.0s      2317 + 399 passed
FIXED     533.8s      2317 + 399 passed
          −59.2s      −10.0%
```

A single test run alone improves far more — `trusted_history_rejects_correction_rollback_and_divergent_same_count`
goes **143.68s → 75.39s (−48%)** — because with one test on the box the git spawns dominate. In
the full parallel suite twenty cores are saturated and the critical path is set by a different,
non-config-bound test, so the suite gains 10% rather than 48%.

**An earlier estimate of −33% is not supported and is not claimed.** That figure came from a
patch that also memoized git *layout* queries; that half was rejected for breaking the
fail-closed stability guard when `.git` is a file (linked worktree, submodule), reproduced 3/3.
Most of the 33% evidently came from the rejected half.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-076 | Four `git config --get` spawns become one `--get-regexp`, taking an instrumented suite run from 15,359 `git config` spawns to 3,842. Equivalence verified against git 2.50.1 across six behaviours including the two that matter for safety — unset (rc=1, empty streams) and malformed (rc=128, stderr) must stay distinguishable. Both new tests and the three existing precedence tests pass identically on a separate-checkout unfixed binary and on this one, which is the correct bar for a refactor. Clean sequential A/B on an idle machine: 593.0s → 533.8s, −10.0%, with 2317 + 399 passing in both |
