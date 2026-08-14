---
change: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: tasks
---

# Tasks

- [x] Record a load failure on the configuration rather than discarding it
- [x] Set it at the unreadable-file sites
- [x] Set it at the parse-failure site — the actual #570 trigger, distinct from the others
- [x] Refuse at the shared entry point every spec-reading command uses
- [x] Name the file and state both ways forward
- [x] Confirm a valid config is unaffected
- [x] Confirm **no** config file is unaffected
- [x] `cargo fmt --all -- --check` clean
- [x] `cargo clippy -- -D warnings` exit 0
- [x] `cargo test` green — 2210 unit, 331 integration, 0 failures
