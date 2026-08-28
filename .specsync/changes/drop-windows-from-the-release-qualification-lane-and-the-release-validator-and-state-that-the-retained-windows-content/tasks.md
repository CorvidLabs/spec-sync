---
change: drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content
artifact: tasks
---

# Tasks

- [x] Remove `windows` from the `qualify` matrix in `.github/workflows/release.yml`
- [x] Remove the Windows-only shell-preparation and Fledge-installation steps, and the `SPECSYNC_BASH` shim
- [x] Drop the now-redundant `runner.os != 'Windows'` guard from the sole remaining Fledge installer
- [x] Set `REQUIRED_PLATFORMS = ("ubuntu", "macos")` and comment the matrix coupling at the constant
- [x] Retarget the validator self-test from `windows` to `macos` and confirm 50/50 pass
- [x] Fix the duplicate dict key the retargeting introduced, which had silently neutered the mixed-evidence test
- [x] Correct the release summary string to say Windows is neither built nor qualified
- [x] Keep #734's `open_specs_directory` gate fix; drop its `windows-check` CI job
- [x] State in `docs/ci-confidence.md` that `#[cfg(windows)]` code is now compiled and run nowhere
- [x] Amend the #722 CHANGELOG paragraph to say its argument was correct and the risk is accepted, not resolved
- [x] Record how to reverse this, and that the validator constant and the matrix must move together
