# Documentation accuracy requirements

## Intent mapping

- DOCS-4: Correct the reference/contributor facts in the audit, retaining explicit version and evidence boundaries. Resolve broken destinations against primary sources or document actual local installation; do not invent availability.
- DOCS-5: Check each diagram against actual current state/transition/workflow definitions. Distinguish SpecStatus from ChangeState and describe source abstractions honestly.
- DOCS-5.a -> REQ-github-026: Required CI parses maintained Mermaid and checks lifecycle correspondence with source; malformed and drifted mutations must fail.
- DOCS-2/DOCS-2.a -> modified REQ-agents-009: Generated and copied closeout guidance agrees on local verification, human review, native archive commit, push, and hosted CI.

Stable module requirements are defined in deltas/agents.md and deltas/github.md. This artifact traces the approved HI to them; it does not allocate a competing set of requirement IDs.

## Completion criteria

Every planned correction cites source or a retained command result, protected hunks remain intact, all generated artifacts match the installer, meaningful focused checks fail before the fixes and pass afterward, and existing verification/trust gates remain unchanged. Native and human closeout gates remain distinct. Any new scope or unmet dependency is surfaced rather than hidden by changing a timeout or weakening an assertion.
