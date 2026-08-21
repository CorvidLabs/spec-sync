---
change: a-reopen-must-extend-the-committed-ledger-not-merely-count-itself
artifact: context
---

# Context

Repairs two defects introduced by the #660 anchor fix, both found by the sandbox after the Rust
suite, a per-risk-class corpus sweep, and three adversarial attack agents had all passed it.

The first is a live laundering hole. The second is a lifecycle regression. They share a cause:
the fix leaned on a number the attacker writes, and switched off the only stage that a reopen
lifecycle ever reaches.

Worth recording plainly, because it is the second time this pattern has cost us. The #660 design
verified its `generation` term as *"dormant today — no archive in the corpus has two
introductions."* Dormant meant untested. A reopen creates the untested state, and only the drills
reopen. The lesson is not "test more"; it is that **"this cannot arise today" is a defect
report, not a safety argument**.
