---
change: CHG-0062-harden-mcp-root-confinement-write-authorization-argument-validation-and-notif
artifact: tasks
---

# Tasks

- [x] Add failing characterization tests for outside-root writes, symlink escape, bad argument
  types, unknown keys, and known notifications.
- [x] Add the `--allow-write` CLI surface and read-only default.
- [x] Implement canonical root confinement and write-root immutability.
- [x] Implement exact tool argument validation and notification response semantics.
- [x] Update MCP, CLI-argument, and root-dispatcher specs, requirements, context, testing, and public
  documentation.
- [x] Run targeted MCP tests (44 unit and 30 integration tests).
- [x] Rerun the complete Rust suite after the final availability hardening.
- [x] Run strict spec, score, and complete repository gates.
- [x] Resolve independent correctness and adversarial security review findings.
