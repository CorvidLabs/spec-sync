---
change: stop-restoring-a-build-cache-in-the-job-that-produces-the-released-binaries-because-save-if-false-proves-the-job-cannot
artifact: context
---

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
