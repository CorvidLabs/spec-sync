---
change: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
artifact: design
---

# Design

## A second question, asked once

`is_git_repo` answers "is this a work tree". Deciding anything from history needs a second
answer — "is there any history" — so `git_utils` gains:

```rust
pub fn has_commits(root: &Path) -> bool
```

implemented as `git rev-parse --verify HEAD`, which fails precisely on an unborn `HEAD`.

It lives in `git_utils` rather than in `stale` because it is not a staleness concept: any
command that compares against history has the same precondition, and the next one should
find it already there rather than re-deriving it.

## Refusing, and saying which

`cmd_stale` guards on `!is_git_repo(root) || !has_commits(root)` and carries the reason
through to output, so the two causes stay distinguishable:

| state | text | machine-readable |
|---|---|---|
| not a work tree | `Not a git repository — staleness detection requires git history.` | `not a git repository` |
| unborn `HEAD` | `Repository has no commits — staleness detection requires git history.` | `repository has no commits` |

Collapsing both into one message would have been less code and worse: a user who ran this
inside a repository they just created needs to know it is the *commits* that are missing,
not the repository.

## Why not report the specs as skipped instead

The alternative was to keep exit 0 and mark each spec not-checked. Rejected: the command's
entire output is a staleness verdict, so there is nothing left to report once history is
absent. Refusing matches what the no-repository case already did, and consistency between
two branches of the same precondition is worth more than a third behaviour.
