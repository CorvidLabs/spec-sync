---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: context
---

# Context

The archive-first CI flow exposed a gap after #1081. In a branch with 442 archived packages and zero active changes, removing REQ-cmd-change-040 gave an empty error and warning list in both strict audit modes. The comparison branch and the JSON reports are preserved in the overnight handoff for #1105.

The first Archive job checks workspace presence using the packaged Action. The downstream required Lifecycle gate checks out full history and runs `change audit --strict`. Exercising that existing audit invocation on an archived tip keeps one implementation of the rule.

This draft proposes enforcement through lifecycle audit. Whether ordinary `specsync check --strict` should also cover projects without an audit gate is a separate scope choice for the maintainer. The proposed shallow-checkout behavior follows the existing #1081 history diagnostics.
