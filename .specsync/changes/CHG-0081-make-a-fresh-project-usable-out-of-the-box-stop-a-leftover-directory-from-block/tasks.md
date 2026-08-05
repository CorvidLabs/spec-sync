---
change: CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block
artifact: tasks
---

# Tasks

- [x] Widen init detection to go.mod, pyproject.toml/pytest.ini and package.json
- [x] Warn at init time when no verification command is detected
- [x] Skip state.json-less directories in both active-change read paths
- [x] Keep every other read error failing closed
- [x] Extract verify_change_locked and document the lock precondition
- [x] Invert sandbox drill 031 to assert the fix
- [x] Run the full Rust suite
