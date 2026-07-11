---
change: CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation
artifact: design
---

# Design

Fail closed at every enforcement boundary. Markdown replacement stops at any equal-or-higher heading. Verification binds the tested working tree and validates the effective contract. Acceptance repeats dependency/conflict/effective-contract gates. Active deltas are topologically ordered. Persisted paths are portable and scope matching uses component boundaries. Policy loading distinguishes absence from invalid content. Concurrency-sensitive writes use an exclusive project lock and atomic replacement.
