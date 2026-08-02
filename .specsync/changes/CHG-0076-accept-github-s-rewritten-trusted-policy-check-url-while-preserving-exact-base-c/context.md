---
change: CHG-0076-accept-github-s-rewritten-trusted-policy-check-url-while-preserving-exact-base-c
artifact: context
---

# Context

## Trigger

PR #491's first non-bootstrap archive-only gate rejected a successful trusted-policy result with
`check details URL is not an exact workflow run`.

## Root cause and invariant

The publisher requested an `/actions/runs/<workflow-id>` details URL, but GitHub persisted the
official Actions check with its canonical `/runs/<check-id>` URL. A display URL is not a stable
provenance field. Acceptance must instead bind the official check and a successful base-controlled
workflow run to the same repository, candidate SHA, trusted revision, workflow path, and PR.

## Scope

This is a protected verifier correction only. It does not weaken the trusted-policy rule, alter the
workflow, or expand CHG-0074's product behavior.
