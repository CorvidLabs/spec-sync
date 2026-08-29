---
change: stop-restoring-a-build-cache-in-the-job-that-produces-the-released-binaries-because-save-if-false-proves-the-job-cannot
artifact: testing
---

# Testing

## What is verifiable

| check | result |
|---|---|
| `release.yml` still parses | `yaml.safe_load` OK |
| No caching step in the `build` job | `grep -n 'rust-cache\|actions/cache@'` returns only line 291, which is `qualify` |
| `qualify`'s cache unchanged | `save-if: false` still at line 301 |
| Rust suite | unaffected — no `src/` change |

## Honest label: no DISCRIMINATOR is possible here, and none is written

The change **removes** a step. There is no assertion that fails against unfixed `main` and passes
here without simply restating the diff — a test that greps the workflow for the absence of
`rust-cache` would be a change-detector, not a regression test, and it would pass for the wrong
reason the moment someone renames the action.

What actually verifies this is the CodeQL rule itself: alert #68 should close on the next scan of
`main`, and re-open if the step returns. That is a real external check with a real oracle, which
is more than a self-authored grep would provide.

**The guard against regression is the comment**, which states that this job must never gain a
caching step and why. That is weaker than a test and is stated as such rather than dressed up.

## What this does not verify

That the cache was ever poisoned, or that any published binary was affected. No evidence of either
exists, and none is claimed. This closes an available path, it does not respond to an incident.
