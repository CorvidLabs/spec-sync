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

CHANGE-7.a.1 and CHANGE-7.a.2 are captured from Leif's answer in the orc tab on 2026-11-05 at 20:42 MST (issue #880, audit D3): "Approve", for these two sentences as worded. Relayed by orc t18; recorded by the ss-s3 Claude agent.

CHANGE-12.d is captured from Leif's answer in the orc tab on 2026-11-06 at 06:1x MST (issue #914): "Ok yes I approve all", in reply to a brief that proposed this sentence as worded. Relayed by orc t18; recorded by the ss-914 Claude agent.

CHANGE-12.e is captured from Leif's answer in the orc tab on 2026-11-06 (issue #960, from the Codex review of #944): "Capture both", confirming this sentence as worded. Relayed by orc t18; recorded by the ss-960 Claude agent.

The retirement of CHANGE-6.b, CHANGE-6.d, and CHANGE-13 through CHANGE-13.d were drafted by an agent for issue #633 (audit D7) and approved by Leif in the Herdr orc tab on 2026-11-05 at 20:19 MST, relayed by orc t18: "Approve", as proposed. On whether the relay record belongs in the review digest he said "Keep it out". Recorded by claude (ss-d7).

CHANGE-14, CHANGE-14.a and CHANGE-14.b were drafted by the agent from the SS-M05 audit findings (a bypassed incomplete-task guard left every selected test passing, and the refusal named no recovery while status repeated the check command). Leif approved them in the Herdr orc tab on 2026-11-05 at 20:20 MST ("Approve", the agent's wording, renumbered from the proposed CHANGE-13), relayed by the orc (t18). Recorded by the ss-testgaps agent.

CHANGE-14.c was drafted by the agent after a fixture showed a task reopened after a passing check being reviewed and finalized (#892). Leif approved it in the Herdr orc tab on 2026-11-05 at 20:24 MST ("Widen S6"), relayed by the orc (t18). Recorded by the ss-testgaps agent.

CHANGE-7.b is captured from Leif's answer in the orc tab on 2026-11-06 at about 17:35 MST (issue #931, from the Codex review of #901): "Capture both", for this sentence as drafted by the agent. Relayed by orc t18; recorded by the t18 worker Claude agent.

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
  - **CHANGE-6.c**  Changing the agreed intent, contract, criteria or scope costs a fresh approval; getting on with the implementation does not.
  - **CHANGE-6.d**  An approval tells me it matches the content it approved and nothing more; the name on it is a claim, not proof of who gave it.
- **CHANGE-7**  At any moment I can ask what state a change is in and be told the single next thing to do.
  - **CHANGE-7.a**  I am told whether it is safe to walk away and clear my head, or what to write down first.
    - **CHANGE-7.a.1**  A change whose archive fails its integrity check is never called safe to walk away from.
    - **CHANGE-7.a.2**  I am told the exact command that restores that archive from trusted history, and once it is restored I am told it is safe again.
  - **CHANGE-7.b**  A name that comes from my repository is shown as plain text: it cannot add lines or terminal commands to what status tells me.
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
  - **CHANGE-12.e**  After I send a change back, it needs a fresh review before it can ship, even when its scope comes back unchanged.
- **CHANGE-13**  When an agent records a decision a person made somewhere else, I can see who decided, their exact words, where they said it, and which agent wrote it down.
  - **CHANGE-13.a**  That record is optional, so a person approving at their own terminal is not asked for one.
  - **CHANGE-13.b**  The change's status shows it beside the approval or review it belongs to.
  - **CHANGE-13.c**  The status says plainly that the record is a claim too, not proof that the person decided.
  - **CHANGE-13.d**  A record missing one of those details is refused before anything is written, and the refusal says which detail is missing.
- **CHANGE-14**  A change cannot pass its check while any of its tasks is still open.
  - **CHANGE-14.a**  The refusal names each open task and the command to run once it is done.
  - **CHANGE-14.b**  The change's status names that same way forward instead of telling me to run the check again.
  - **CHANGE-14.c**  A task reopened after a passing check stops the change from being reviewed or finalized until it is done again.

## Retired

  - **CHANGE-6.b**  A coding agent can never grant that approval to itself.
    retired: An approval records the name it is given; nothing local can tell a person from an agent typing that name (audit D7, #633). Replaced by CHANGE-6.d.
