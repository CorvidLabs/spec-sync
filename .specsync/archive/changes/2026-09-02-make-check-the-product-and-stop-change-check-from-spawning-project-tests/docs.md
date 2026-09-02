---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: docs
---

# Docs

User-facing docs now say `change check` applies deltas and compares specs to code. It does not
run the project's tests. `verification_commands` remains on the policy file for adopters who
still list them; `change check` does not execute the list. CI owns `cargo test`.

Surfaces updated: `site/src/content/docs/workflow.md`, `quickstart.md`, `configuration.md`,
`deltas.md`, `AGENTS.md`, and generated agent skills/commands.
