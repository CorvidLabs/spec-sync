---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: context
---

# Context

Issue #893: `change check` and `change audit` do not execute configured verification commands, but a passing result gives an adopter no visible reminder of that limit. SHIP-1.c.3 asks both commands to disclose what they did not run so a pass is not mistaken for the project's tests passing.

The committed integration probes failed first on b3ca5f97, recorded as OBS-product-test-693137d491cd153adf290ec2. They cover plain and committed checks, audit, JSON, an empty policy, singular wording, no process spawn, and this repository's policy.

Leif first approved emptying this repository's top-level `verification_commands`. His narrowed 3b decision, relayed by orc t18 on 2026-11-06, is authoritative: empty only that list; keep all 13 component modules and all 3 strict commands. Emptying the other lists to silence the note is rejected. The repository contract pin has been narrowed accordingly and still fails only on the four top-level commands.
