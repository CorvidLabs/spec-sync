---
change: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
artifact: testing
---

# Testing

## Discrimination

The two disclosure assertions were run against an unfixed binary built from a
**separate checkout** at `adbfb442` — not by reverting files — with the new test
file copied in and its `mod` declared. Confirmed absent there:
`grep -c resolve_watch_dirs src/watch.rs` → 0.

| Test | unfixed `adbfb442` | fixed |
|---|---|---|
| `watch_warns_on_nonexistent_directory` | FAILED | ok |
| `watch_warns_on_nonexistent_directory_json` | FAILED | ok |
| `watch_does_not_claim_a_pass_over_zero_specs` | FAILED | ok |
| `watch_errors_when_all_directories_missing` (control) | ok | ok |
| `watch_still_reports_a_pass_over_a_real_spec_set` (control) | ok | ok |

The two controls pass on both binaries, so neither "refuse every directory" nor
"never report a pass" can satisfy the change.

## Evidence per requirement

| Requirement | Evidence |
|---|---|
| REQ-watch-002 | `resolve_watch_dirs` returns `watched` and `skipped` instead of filtering; three unit tests cover all-present, all-missing, and the partial case; two integration tests assert the human and JSON disclosures name the configured path |
| REQ-cli-009 | `run_check` tracks whether the child reported finding no specs and reports that instead of a pass; `reports_no_specs` has a positive test and a four-line negative control so a real run is never mistaken for an empty one |

## Sandbox

Drill 060 (`060-watch-dropped-dir.sh`) is the judge — it observes the real
process, which the single-process Rust suite cannot. Against the fixed binary:

```
- PASS: check --strict fails the same tree (control: #560/#582 still hold)
- PASS: watch produced an initial run rather than hanging mute
- PASS: watch disclosed the missing configured specs_dir next to the banner
- PASS: watch did not park All checks passed! over an empty spec set
- PASS: valid config still watches specs and reports a real pass (control)
- PASS: empty watch set still fails closed (control)
- PASS: harness output stayed outside every tree under test
pass=7 fail=0 pending=0
verdict: PASS
```

Drill 060 is a gate drill (>= 044) and self-flips; no inversion commit is needed.
