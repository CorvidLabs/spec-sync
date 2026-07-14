---
change: CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps
artifact: context
---

# Context

PR #370 is otherwise green and release-ready, but its final review found two lifecycle enforcement gaps: Cargo commands selecting a nested SpecSync manifest can evade recursive-verifier detection, and implicit affected-spec coverage excludes standard companions beyond `requirements.md`. A separate inspection of the 5.x interview found that all answers are currently split on commas and newlines, which corrupts ordinary prose acceptance criteria.

These are 5.0.2 integrity and intent-preservation fixes, not the broader 5.1 guided-UX work. The implementation stays inside the existing lifecycle model and must not weaken explicit scope, registry authority, verification isolation, or human approval gates.
