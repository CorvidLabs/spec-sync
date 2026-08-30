---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: tasks
---

# Tasks

- [x] Stop `specsync check` from calling `audit_project`
- [x] Write SDD off on fresh `init`
- [x] Replace `verify_change` command spawn with in-process spec↔code sync
- [x] Stop `change audit` from executing `verification_commands` in CI
- [x] Discriminator: configured sentinel is not spawned
- [x] Control: phantom export fails `change check`
- [x] Rewrite REQ-change-023/049/050/058/091 and cmd_check/cmd_init/cmd_change/agents specs
- [x] Cover dirty paths with this change workspace so `change audit --strict` can pass
