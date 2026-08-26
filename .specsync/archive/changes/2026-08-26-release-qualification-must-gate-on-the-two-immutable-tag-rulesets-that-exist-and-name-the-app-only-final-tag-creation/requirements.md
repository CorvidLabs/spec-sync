---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: requirements
---

# Requirements

REQ-github-007 is modified, not replaced. Its binding of Ubuntu/macOS/Windows evidence to one
immutable candidate SHA is unchanged; only the tag-protection acceptance criteria move.

**Was** (one criterion):

> Active tag rulesets let humans create new RC markers but forbid their update/deletion, allow only
> a dedicated release GitHub App to create final tags, and forbid every actor from updating or
> deleting final tags. Its private key is available only to the protected `release` environment's
> promotion job, which mints a short-lived token scoped to the repository.

**Now** (two criteria):

> - Two active tag rulesets let humans create new RC markers and final tags but forbid every actor,
>   with no bypass, from updating or deleting either. Qualification validates exactly those two —
>   `SpecSync immutable RC tags` over `refs/tags/v*.*.*-rc.*` and `SpecSync immutable final tags`
>   over `refs/tags/v*.*.*` excluding the RC pattern — and fails closed on any broadening.
> - Final-tag creation is not restricted to a release GitHub App and the protected `release`
>   deployment environment is not validated. Qualification states both omissions on every run,
>   including successful ones, and fails if that statement is ever empty; it never reports a
>   protection it does not check.

What each half does:

- The first criterion is narrower in **count** (two rulesets, not three) and stricter in **kind**
  (no bypass actor is admissible anywhere; previously one ruleset admitted an `Integration`).
- The second criterion is new obligation, not a waiver. It is what stops this from being a silent
  weakening: the requirement is now that the gap be **stated**, and stating it is enforced by
  failing when the statement is empty.

The Invariants and Error Cases sections of `specs/github/github.spec.md` carry the same rule in
durable form: a gate that demands an unprovisioned policy fails on every candidate and therefore
verifies nothing, so announcing a gap is the safe default and demanding one is not.
