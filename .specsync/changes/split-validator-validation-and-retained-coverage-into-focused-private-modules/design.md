---
change: split-validator-validation-and-retained-coverage-into-focused-private-modules
artifact: design
---

# Design

Keep validator.rs as the stable API facade, including SourceSnapshot and reference/dependency entry points as appropriate. Private validation.rs owns the complete ordered core; validation/policy.rs owns companion markers and custom rules; schema_checks.rs owns schema resolution; mappings.rs owns source-path checks and suggestions. Private coverage.rs owns orchestration and shared budgets, with configuration.rs, sources.rs, specs.rs, access.rs, ownership.rs and test_barriers.rs children. Access owns retained identities and platform-specific no-follow operations. Children borrow existing state rather than recreate authority or accounting. Use explicit pub(super) seams and preserve existing public/crate-visible API paths via wrappers or re-exports; no additional crate-visible interface. Topical child tests preserve all test bodies and platform guards.
