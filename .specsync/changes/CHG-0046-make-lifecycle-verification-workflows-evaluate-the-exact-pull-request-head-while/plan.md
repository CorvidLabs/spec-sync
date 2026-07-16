---
change: CHG-0046-make-lifecycle-verification-workflows-evaluate-the-exact-pull-request-head-while
artifact: plan
---

# Plan

1. Record and approve this no-spec-change operations definition through supported SpecSync
   commands.
2. Configure exact-head checkout with full history in only the CI spec-check and Trust jobs.
3. Add a deterministic workflow-structure regression assertion if the repository's test
   surface can express it without coupling to YAML scalar implementation details.
4. Validate workflow parsing, strict SDD coverage, affected lifecycle evidence, and Trust at
   one common head.
5. Stop at a clean pre-push checkpoint so the independent Windows correction can be combined
   before final hosted validation.
