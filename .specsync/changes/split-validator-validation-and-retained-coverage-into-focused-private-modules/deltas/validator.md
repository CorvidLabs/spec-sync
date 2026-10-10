## ADDED

### SPEC SECTION Internal Organization

The validator keeps its existing public and crate-visible entry points behind a small facade. Private responsibility modules own ordered single-spec validation, companion/custom-rule policy, schema identity checks and source mapping helpers. Retained coverage has private owners for orchestration and shared budgets, configuration, source traversal, spec inventory, capability access, module ownership and debug race barriers. Existing tests are grouped beside their owners. Module boundaries preserve all API paths, diagnostic and traversal ordering, exact-byte snapshot authority, retained identities, shared limits, manifest degradation notices and platform guards; they introduce no new externally visible behavior.
