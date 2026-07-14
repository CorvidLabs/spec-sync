---
change: CHG-0031-keep-the-rustsec-audit-job-compatible-with-the-rust-1-89-msrv
artifact: tasks
---

# Tasks

- [x] Replace the unbounded audit action installer with a pinned locked Cargo installation.
- [x] Validate the updated workflow with `actionlint`.
- [x] Keep the audit result mandatory in the aggregate required CI gate.
