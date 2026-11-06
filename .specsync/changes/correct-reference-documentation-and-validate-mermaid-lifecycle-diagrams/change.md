---
id: correct-reference-documentation-and-validate-mermaid-lifecycle-diagrams
state: draft
type: documentation
base_commit: 7ae7245f97e7cfd66cd80ce39b611becb1095642
---

# Correct reference documentation and validate Mermaid lifecycle diagrams

## Intent

Correct reference documentation and validate Mermaid lifecycle diagrams

## Affected Canonical Specs

- `agents`
- `github`

## Acceptance Criteria

- DOCS-4: Reference and contributor documentation accurately describes current commands, settings, defaults, paths, supported platforms and links.
- DOCS-5: All maintained diagrams reflect current implementation states, transitions and steps.
- DOCS-5.a: CI checks every maintained Mermaid block for parser validity and checks lifecycle states and send-back transitions against source, with failing mutation controls.
- DOCS-2 and DOCS-2.a: Reference and generated/copied skill closeout guidance agrees on local verify, review, native archive commit, push and CI.
- Protected open-PR lines remain unchanged; HLD F22 follows #935 and Chough session records follow #941. One docs-focused PR carries tests, generated prose and contracts without changing executable lifecycle behavior.

## No-spec Rationale

Not applicable
