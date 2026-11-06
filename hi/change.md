---
hi: 1
families: [CHANGE]
---

# Agreeing a change

## Intent

Before anyone writes code, the change should be something a person agreed to in writing. Opening one asks a short, predictable set of questions instead of handing over a blank template, and the contract edit travels with it as a delta anyone can read. Exactly one human approval binds that agreed scope to its content, so an agent cannot quietly widen what was agreed and a later reader can see what was actually promised.

CHANGE-10 is captured from Leif's ask in Grok Bot chat t811u on 2026-11-03: "`change approve` refuses affected paths that no module owns (for example a new module's own `.spec.ts` files), with a fix that requires editing the module's files list; say which spec to edit in the error."

CHANGE-10.a, CHANGE-10.b and CHANGE-10.c quote, verbatim, the optional nits in Leif's GitHub review of #841 (review 5771062363, APPROVED, 2026-11-04 08:33 MST). On 2026-11-04 at 09:25 MST Leif said: "Yes, merge #841 and open the nits follow-up PR."

CHANGE-11 is captured from Leif's answer in a Grok Bot widget on 2026-11-04 at 7:34 AM MST (issue #843): "Confirm #843 criteria as written", choosing to "Protect only changes approved from now on". Recorded by corvid-agent.

CHANGE-12 is captured from Leif's answer in a Grok Bot widget on 2026-11-04 at 7:34 AM MST (issue #844): he chose "Both A and B" for the options, which confirms option A (send a change back for re-scoping) with these criteria. Recorded by corvid-agent.

CHANGE-12.d is captured from Leif's answer in the orc tab on 2026-11-06 at 06:1x MST (issue #914): "Ok yes I approve all", in reply to a brief that proposed this sentence as worded. Relayed by orc t18; recorded by the ss-914 Claude agent.

## Criteria

- **CHANGE-1**  I can open a change from a plain sentence describing what I mean to do.
  - **CHANGE-1.a**  The change takes a readable identity from that sentence rather than a number I have to coordinate with anyone.
  - **CHANGE-1.b**  Opening a change that repeats one already in flight is refused.
  - **CHANGE-1.c**  That refusal names the change already holding the description.
- **CHANGE-2**  Opening a change asks a short, predictable set of questions instead of handing me a blank template.
- **CHANGE-3**  The documents a change must produce follow its risk, so a typo fix does not drag a design document behind it.
- **CHANGE-4**  A change says up front what it is allowed to touch.
  - **CHANGE-4.a**  It names the contracts it will alter.
  - **CHANGE-4.b**  It names the paths in the code it will touch.
  - **CHANGE-4.c**  A change that claims it alters no contract has to say why.
- **CHANGE-5**  The contract edit travels with the change as a delta I can read: what is added, what is changed, what is retired.
  - **CHANGE-5.a**  Every new or changed requirement points at the evidence that will prove it.
  - **CHANGE-5.b**  Two live changes editing the same part of the same contract is a conflict I hear about early, not at merge.
- **CHANGE-6**  Exactly one human approval stands between an agreed change and the work starting.
  - **CHANGE-6.a**  That approval is bound to the content it approved, so later edits cannot ride on it.
  - **CHANGE-6.b**  A coding agent can never grant that approval to itself.
  - **CHANGE-6.c**  Changing the agreed intent, contract, criteria or scope costs a fresh approval; getting on with the implementation does not.
- **CHANGE-7**  At any moment I can ask what state a change is in and be told the single next thing to do.
  - **CHANGE-7.a**  I am told whether it is safe to walk away and clear my head, or what to write down first.
- **CHANGE-8**  I can see every change in flight at once, without opening them one by one.
- **CHANGE-9**  I can declare that one change has to land before another, so the order is not folklore.
- **CHANGE-10**  When `change approve` refuses an affected path that no module owns, the error says which spec to edit.
  - **CHANGE-10.a**  `ownership_hint` picks the first declared spec even at depth 0, so `lib/new.ts` can be pointed at `specs/billing/billing.spec.md`. Guard on `depth > 0` and fall back to the generic message.
  - **CHANGE-10.b**  The first-pass walk returns the first undeclared spec that lists the path, in `read_dir` order. That order isn't deterministic across platforms. Collect every match and sort.
  - **CHANGE-10.c**  Missing tests: the depth-0 fallback, `src/foo` vs `src/foobar`, a glob entry, and the `_ =>` arm.
- **CHANGE-11**  A change approved against older contract text never silently overwrites newer text another change put there.
  - **CHANGE-11.a**  Checking that change stops and names the requirement or section whose text moved.
  - **CHANGE-11.b**  The project health check (audit) reports the same problem before anything is lost.
  - **CHANGE-11.c**  The message tells me the way forward: rewrite the change onto the current text and approve it again.
- **CHANGE-12**  When a change's scope turns out wrong after approval, I can send it back to be re-scoped instead of deleting it by hand.
  - **CHANGE-12.a**  Sending it back needs a named person and a reason, and both stay in the change's history.
  - **CHANGE-12.b**  Sending it back cancels its approval, so the new scope needs a fresh one.
  - **CHANGE-12.c**  Nothing already recorded about the change is lost.
  - **CHANGE-12.d**  A change I send back after it was reviewed can be reviewed again once it is approved again, and the earlier review stays in its history.
