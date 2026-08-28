---
change: drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content
artifact: testing
---

# Testing

## What is verifiable

| check | result |
|---|---|
| Validator self-test | `python3 .github/scripts/test-validate-release-candidate.py` — **50 passed** |
| `release.yml` still parses | `yaml.safe_load` OK |
| No Windows-only step survives | `grep -in windows .github/workflows/release.yml` returns only the summary line stating Windows is not qualified |
| No Windows reference survives in the validator or its test | `grep -ci windows` → 0 in the test file; the constant carries an explanatory comment |
| Rust suite unaffected | the only `src/` change is a `cfg` attribute on a test-only helper |

## A test that caught a real defect in this change

Retargeting the validator's self-test from `windows` to `macos` was done by textual replacement, and
in `test_release_refuses_missing_or_mixed_evidence` that produced a **duplicate dict key**:

    "ubuntu": {"workflow_revision": "a" * 40},
    "macos":  {"workflow_revision": "b" * 40},
    "macos":  {"workflow_revision": "a" * 40},

The second entry silently overwrote the first, so the "mixed workflow revisions" case stopped being
mixed and the test passed for the wrong reason — a green run asserting nothing. Caught because the
suite failed on `assertNotEqual(returncode, 0)` rather than because anyone read the diff.

**Honest label: CONTROL.** That test must keep failing the validator on genuinely mixed evidence. If
a future edit makes its two platform records agree, it becomes vacuous again and will pass silently.

## What cannot be verified any more, by design

Everything `#[cfg(windows)]`. No job in this repository compiles it. That is the accepted cost of
this change and is stated in `docs/ci-confidence.md` and the CHANGELOG rather than left implicit.
There is no test that can assert an absent guarantee; the disclosure is the mitigation.
