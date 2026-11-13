---
change: the-documented-workflow-test-requires-the-quick-start-fill-in-step-before-the-first-strict-check
artifact: context
---

# Context

Issue #968, from the Codex review of #952. #906 added `# fill in specs/auth/ ...` to the
landing-page Quick Start right after `specsync add-spec auth`. The documented-workflow test
checked only that the line comes after `add-spec`, not that it comes before the first
`specsync check --strict`. If a later edit moved the line below the strict check, the test would
still pass, and readers would run strict validation on the untouched scaffold, which it refuses.

The ordering check moves into `quick_start_fill_in_position`, which the fixture setup and a
mutation test both call. The mutation test moves the fill-in line below `specsync check --strict`
and expects a refusal. The Quick Start's other lines are a command list the harness does not
run, so the fixture still writes the filled-in scaffold where it did; the ordering assertion is
what the issue asks for at minimum.
