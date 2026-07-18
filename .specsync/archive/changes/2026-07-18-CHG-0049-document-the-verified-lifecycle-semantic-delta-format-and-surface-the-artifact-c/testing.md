---
change: CHG-0049-document-the-verified-lifecycle-semantic-delta-format-and-surface-the-artifact-c
artifact: testing
---

# Testing

Pre-acceptance verification will run:

1. `cd site && bun test` for documentation-site behavior.
2. `cd site && bun run lint` for Astro and content linting.
3. `cd site && bun run build` to resolve internal links and produce the deployable site.
4. `specsync check --strict --require-coverage 100 --force` to prove both meaningful documentation paths have valid lifecycle ownership and the repository remains fully covered.
5. `fledge trust verify` after the repository verification lane passes, treating any Augur block verdict as a hard stop.

Manual review will compare the documented grammar, evidence rules, ordering, and acceptance behavior with `src/change.rs` and the executable lifecycle examples.
