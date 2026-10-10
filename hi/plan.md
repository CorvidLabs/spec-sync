---
hi: 1
families: [PLAN]
---

# Plan

## Intent

Opening a change should show me what the modules already learned and the house principles, and the plan should say what I will and will not do before anyone writes code. A lesson becomes a rule only when I write it down myself. Merging still does not wait on copying lessons into the spec.

## Criteria

- **PLAN-1**  When I open a change, I can see the lessons already written for the modules it touches, and the project's principles when I have any.
  - **PLAN-1.a**  My agent sees the same thing in the structured result.
  - **PLAN-1.b**  A module that still has only its original scaffold is not presented as if it had learned something.
- **PLAN-2**  A change that needs a plan cannot be approved until that plan says what I will do, what I will not do, the steps, the risks, and which existing constraints I am honoring.
  - **PLAN-2.a**  A change the interview did not select a plan for still does not grow one.
- **PLAN-3**  While a change is still a draft, and only when lessons or principles exist, I am asked which of them constrain it, and "none" is a complete answer.
  - **PLAN-3.a**  Changing that answer is a change of the agreed scope.
- **PLAN-4**  When a change is archived I am shown where its lesson bundle is, and I can still merge without copying those lessons into the spec.
- **PLAN-5**  A lesson becomes a standing rule only when I write it into the project's principles, and the next change shows me that file before I approve it.
