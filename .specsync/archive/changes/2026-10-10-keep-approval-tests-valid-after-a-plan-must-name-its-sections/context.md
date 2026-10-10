---
change: keep-approval-tests-valid-after-a-plan-must-name-its-sections
artifact: context
---

# Context

Two approval tests still treated a one-line plan as complete, and the principles test never answered the new constraints question. Both failed in CI after the plan gate and the constraints question landed. The tests now fill a real plan, and the principles test records `none` before approval.
