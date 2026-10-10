## ADDED

### SPEC SECTION Internal Organization

Manifest discovery keeps its existing public and crate-visible API paths behind a small facade. Private ecosystem parsers share retained access and native identity helpers. Gradle discovery separates orchestration, bounded retained reads, syntax and executable-directive guards without changing their invocation order or literal interpretation. Existing tests are grouped beside their implementation owners. The split preserves fixed parser order, first-name-wins merging, diagnostics, capability authority, workspace budgets and memoization, bounded handle lifetimes, all guards and platform behavior; it introduces no new public behavior.
