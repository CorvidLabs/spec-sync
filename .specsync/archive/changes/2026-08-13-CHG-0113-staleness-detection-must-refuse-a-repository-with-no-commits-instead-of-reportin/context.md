---
change: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#558, found by sweeping affirmative-claim sites rather than from a
report.

```
$ specsync stale        # git repo, zero commits
  Result:    0/1 specs are stale
  ✓ All specs are up to date with their source files.
exit 0
```

Nothing was compared. Staleness is decided against git history, and an unborn `HEAD` has
none — no commit exists for a spec or its source to be newer or older than.

## Why it matters

`stale` answers "has my code moved on without its spec". An affirmative "up to date" when
it could not look is worse than no answer, because it is indistinguishable from a real one.

The zero-commit state is not exotic: it is exactly what `git init` plus `specsync init`
produces — where the quick start begins — and it is common in CI for a freshly created
repository or a checkout that fetched no history.

## The shape of the miss

The no-repository case was **already correct**:

```
Error: Not a git repository — staleness detection requires git history.
exit 1
```

So "no history to work with" had been considered and handled. An unborn `HEAD` simply was
not recognised as an instance of it: `is_git_repo` answers "is this a work tree", which is
true, and nothing asked the second question.

That is worth recording, because it is the same shape as #560 — the requirement was
understood and written down, and one instance of it slipped through.

## What a session picking this up needs to know

`diff` is the model for this class. Faced with a degenerate comparison base it says:

> no PR base detected … In a clean CI checkout this compares nothing — pass `--base <ref>`
> to choose a real comparison base.

It names the situation, says what it means, and says what to do. Every command that decides
something from history should read like that.

This is the seventh instance of one class this cycle (#546, #547, #548, #550, #553, #558,
#560): a category empty for want of *input*, read as want of *problems*.
