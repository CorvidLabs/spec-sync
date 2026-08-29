# Lesson bundle — stop-restoring-a-build-cache-in-the-job-that-produces-the-released-binaries-because-save-if-false-proves-the-job-cannot

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Stop restoring a build cache in the job that produces the released binaries, because save-if false proves the job cannot poison the cache and not that its output is trustworthy
- **Kind**: Operations
- **Specs**: github
- **Paths**: .github/workflows/release.yml, CHANGELOG.md
- **Acceptance**: The release build job contains no caching step and a comment stating why it must never gain one; the qualify job's cache is unchanged and the reason for leaving it is recorded; release.yml still parses; and CHANGELOG.md states what save-if false does and does not establish, and that two sibling CodeQL alerts were dismissed as already mitigated while this one was not.

## Evidence

- Verification commit: `a85052a7802d51be1aa1ad2d4ace4eb33edaa4b2`
- Base commit: `db1f4ac95d0a81eecb1777d351a52222fb1aa75f`
- Verified by: `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

## What led here

CodeQL raised four alerts on `main`. Three were dismissed after inspection; this one was not,
because it was real.

- **#63** (`authorize-release`) and **#65** (`validate`) — dismissed. Neither job has a caching
  step at all, so there is no cache for an untrusted checkout to poison. `authorize-release`
  already carries a guard comment requiring `save-if: false` if one is ever added.
- **#67** (`rust/cleartext-logging` in `src/change_tests.rs`) — dismissed. The file is behind
  `#[cfg(test)]` and never reaches a shipped binary; the printed value is a lifecycle diagnostic
  that REQ-change-056 requires to be generic and to carry no ledger bytes or digest material.
- **#68** (`build`) — **fixed here.**

## The reasoning error being corrected

The removed comment said:

> Restore, never save. […] Writing a cache entry from that tree is what lets a candidate that is
> not what the operator believes seed a cache later restored by default-branch workflows.
> **Reading one is harmless.**

The first half is correct. The last sentence is asserted, not argued, and it is wrong for this job.

`save-if: false` establishes that the job **cannot poison** the cache. It establishes nothing
about whether the job's **output is trustworthy**. `Swatinem/rust-cache` restores `~/.cargo` and
`target/`; a default-branch cache poisoned by any other route would be linked into the published
artifacts, and cargo would reuse prebuilt objects out of it.

This is the job whose output is signed, published, and installed by other people. For it, the
question that matters is the second one.

**This is the release's recurring defect shape, in our own security reasoning.** A question that
was not asked produced no negative answer, and the silence read as a pass — the same shape as
#672, #684, #689's first design, #720, #728 and #741. Finding it in a mitigation comment written
specifically to reason about this attack is the point worth remembering: the comment was thorough
about the half it considered.

## What is deliberately NOT done

**The `qualify` job keeps its cache**, with `save-if: false`. Left alone on purpose, and worth a
separate decision rather than a silent extension of this change:

- A poisoned cache there **cannot reach a published binary** any more, because `build` is now cold.
- It **could** make a candidate qualify that should not have — qualification is a verdict, and a
  verdict influenced by a poisoned cache is not trustworthy either.
- The cost differs sharply: `build` runs a handful of times per release, `qualify` runs on every
  release candidate across two platforms.

Recorded in the CHANGELOG so the open question is visible rather than implied by the asymmetry.

## Ruled out

- **Suppressing the alert instead.** The mitigation the suppression would have cited does not
  cover the restore path, so the suppression would have been false.
- **Dismissing #68 alongside its siblings.** My first assessment did exactly that, and reading the
  rule's own recommendation — *avoid caching in workflows that handle sensitive operations like
  releases* — is what caught it. Recorded because the near-miss is the useful part.

## From the change's design.md

# Design

Recorded in `context.md` and `testing.md`: the reasoning error being corrected (`save-if: false`
proves the job cannot poison the cache, not that its output is trustworthy), why the `build` job
is the one that matters (its output is signed, published and installed by other people), why the
three sibling CodeQL alerts were dismissed rather than fixed, why `qualify` deliberately keeps its
cache and what open question that leaves, and why no discriminating test is possible for the
removal of a step.

## From the change's testing.md

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

## Where these lessons go

- `specs/github/context.md`
