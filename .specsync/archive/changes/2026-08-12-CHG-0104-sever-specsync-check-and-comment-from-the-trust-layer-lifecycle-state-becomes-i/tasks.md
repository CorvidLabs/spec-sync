---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: tasks
---

# Tasks

- [x] `check.rs`: informational lifecycle summary, no exit-code contribution
- [x] `check.rs`: shape warnings retained on stderr
- [x] `comment.rs`: stop merging SDD errors/warnings
- [x] `CHANGELOG.md`: document the 1 -> 0 exit-code change for trust-red repos
- [x] Semantic deltas for `cmd_check` and `cmd_comment`
- [x] Sandbox drill 038 (drift invariant) still passes
- [x] Sandbox drill 028 (happy path) still passes
- [x] `change.rs`: delete `check_project_quiet` and the `ConfiguredCommandOutput` vestige
- [x] `cargo clippy -- -D warnings` clean (the gate that caught the orphan)
- [x] Delete the three integration tests that assert the removed behavior
- [x] Full suite green after deletion (2197 unit + 331 integration)
