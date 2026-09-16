---
hi: 1
families: [CHANGE]
---

# Agreeing a change

## Intent

Before anyone writes code, the change should be something a person agreed to in writing. Opening one asks a short, predictable set of questions instead of handing over a blank template, and the contract edit travels with it as a delta anyone can read. Exactly one human approval binds that agreed scope to its content, so an agent cannot quietly widen what was agreed and a later reader can see what was actually promised.

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
