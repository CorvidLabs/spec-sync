---
change: bypass-actors-are-unobservable-to-a-workflow-token-so-absence-must-read-as-unverified-rather-than-empty
artifact: testing
---

# Testing

Three payloads, because the risk is trading one silent hole for another:

- `test_absent_bypass_actors_reads_as_unverified_not_as_empty` — the regression. The field is
  deleted from both fixtures, mimicking what a non-admin token receives. Validation passes and
  emits exactly two notices naming the rulesets that were not checked.
- `test_a_visible_bypass_actor_is_still_refused` — **honest label: the CONTROL.** Softening absence
  must not soften presence. This is what fails if the check were relaxed into accepting any value.
- The existing accept test covers the third case: an admin payload with no bypass actors passes
  and emits no such notice.

Also run against the **live rulesets** rather than fixtures only: the real payloads pass, the same
payloads with `bypass_actors` stripped pass with notices, and a payload granting bypass is refused.

50 validator tests pass.
