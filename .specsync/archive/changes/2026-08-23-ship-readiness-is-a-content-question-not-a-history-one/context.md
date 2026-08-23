---
change: ship-readiness-is-a-content-question-not-a-history-one
artifact: context
---

# Context

#689. Verification evidence is recorded against a commit hash and `ship-status` checked it with
`merge-base --is-ancestor`. A squash-merge rewrites that hash, so a change read as unfinalizable
the moment its own PR landed.

## Not an adopter's configuration

spec-sync's own repository is squash-only:

    {"merge":false,"rebase":false,"squash":true}

Measured on its own history: **19 of 172** archived changes have a reachable verification commit —
11% — and 3 of 489 commits on `main` are merges. The ancestry guarantee has effectively never held
for this project. It was filed as "incompatible with squash-only repositories" from inside one.

## The fix was already written, and one caller missed it

This is the finding that made the change small. `change.rs` removed the ancestry walk from its
currency paths long ago, with the reasoning recorded inline at two sites:

    // Commit identity and ancestry are deliberately not checked here ... binding evidence to a
    // commit ... is a history-trust question rather than a content one. It is also what made
    // squash-merged changes permanently unfinalizable.

    // The git-ancestry walk that used to follow ... answered a different question ... That is
    // `attest`'s job ... Its side effect was the documented deadlock where the lifecycle
    // instructed an author to make a commit its own gate then refused.

`verification_is_current` has answered the content question ever since. `ship_status_report` was
the one caller that never got the change, so it kept asking about reachability.

I came within one compile error of adding a **fourth** resolver beside three that already agree —
`error[E0428]: the name verification_is_current is defined multiple times`. The function I needed
was one line from where I was about to write it. That is this codebase's recurring failure mode
and it nearly claimed the fix for it.

## Why this is safe, from the adversarial survey

`verification.commit` ancestry is **not load-bearing for trust**. It is a self-declared string that
is never resolved to a tree and never compared against `workspace_digest`; it is checked for
membership of a set — ancestors of HEAD — whose contents the attacker chooses. Every tamper the
survey constructed is refused by other guards; the one forgery it built passes the ancestry check
outright.

Two things the survey flagged as genuinely load-bearing, both untouched here:

- ancestry over commits **discovered from history** (`admissible_archive_introductions`,
  `anchor_precedes_an_introduction`) orders two byte-identical introductions of the same package.
  Only history separates them; no digest can.
- the scoped-review descendant walk carries a path allowlist covering `.specsync/changes/` and
  `.specsync/archive/` — precisely the region `project_input_is_volatile` excludes from
  `workspace_digest`. That one is not costume, and it is why `ship` is still gated. See below.

## Deliberately incomplete

`ship` still refuses with "independent scoped review is stale". That gate's ancestry walk guards
the region the content digests do not cover, so removing it the way this change removes the other
would open a real hole. It needs its own design: a squash makes the walk impossible, so it fails
closed forever, and the honest fix states that the history guarantee is unavailable rather than
silently dropping it. Filed separately.
