---
change: release-qualification-must-gate-on-the-two-immutable-tag-rulesets-that-exist-and-name-the-app-only-final-tag-creation
artifact: docs
---

# Docs

`docs/ci-confidence.md` claimed "Final-tag authority is split across three active rulesets… Only
the dedicated CorvidLabs release GitHub App may create a final tag… The App's repository-scoped
token is minted only inside the protected `release` environment, whose deployment policy admits
`main` only." Every clause after the first was false: there is no third ruleset, no App, and no
`release` environment. A doc claiming three rulesets where two are enforced is precisely the drift
this project exists to catch.

Replaced with a `## Tag authority: what is enforced, and what is not` section that:

- tabulates the two rulesets actually enforced, with their exact include/exclude patterns, rules,
  and empty bypass-actor lists;
- states that `resolve` refuses any broadening, so the gate cannot widen without failing;
- lists the two protections deliberately **not** enforced, marked NOT, with the consequence spelled
  out — any actor with tag-write access can create `refs/tags/vX.Y.Z` without a qualified
  candidate;
- distinguishes the App as promotion's *mechanism* from an App-only *policy* nobody enforces;
- explains why: nothing was provisioned, so the check failed on rc.1 through rc.6 and never once
  passed, and a gate that always fails verifies nothing.

`specs/github/tasks.md` had an open item saying "the dedicated release App and production rulesets
still require provisioning and live proof". Half of that is now done and half is now abandoned, so
it is split into the two immutability rulesets (provisioned, live-proven, ids recorded) and the
App/environment decision that remains open.
