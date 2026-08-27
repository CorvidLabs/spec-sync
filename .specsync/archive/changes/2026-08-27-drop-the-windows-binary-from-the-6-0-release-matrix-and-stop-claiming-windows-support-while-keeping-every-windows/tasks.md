---
change: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
artifact: tasks
---

# Tasks

## Stop producing the executable

- [x] `rc-assets.yml`: drop the msvc matrix entry, the pwsh `Package (Windows)` step, the
      `runner.os != 'Windows'` guard, and the `.zip` upload globs
- [x] `rc-assets.yml`: correct the "six targets" / "six build jobs" header comment
- [x] `release.yml`: the same four removals in `build`
- [x] `release.yml`: collapse the `.zip` / `.tar.gz` suffix fork in `Verify packaged checksum`
- [x] `release.yml`: remove `artifacts/**/*.zip` from `Create release` (`fail_on_unmatched_files`)
- [x] `validate-release-candidate.py`: drop the Windows entry from `EXPECTED_ARTIFACT_ARCHIVES`
- [x] `test-validate-release-candidate.py`: repoint the one literal artifact name

## Stop claiming it exists

- [x] `action.yml`: refuse a Windows runner with a WSL message; collapse the zip download fork
- [x] `README.md`: pre-built binaries line
- [x] `github-action.md`: binaries table row, smoke-test sentence, `windows-latest` matrix example
- [x] `quickstart.md`: download note
- [x] `adversarial-proof.md`: CI platform claim
- [x] `CHANGELOG.md`: `### Removed` entry under `[Unreleased]`

## Keep the guarantees from narrowing with the shipped set

- [x] `REQ-change-083` / `REQ-change-084`: rebind to host platforms, not shipped platforms
- [x] `REQ-commands-013`: same
- [x] `commands.spec.md` `is_reserved_module_name` description: same
- [x] `src/commands/mod.rs` doc comment: same
- [x] `view.spec.md` Invariant 8 + `view/context.md`: name Windows, record it is no longer shipped
- [x] `specs/github` requirements/testing/tasks/context: Action consumers pass on Linux and macOS
- [x] `specs/cli` + `specs/cmd_migrate` `## Constraints`: separate shipped set from portability mandate

## Verify

- [x] `cargo fmt` and `cargo clippy -- -D warnings` (clippy is not in `verification_commands`)
- [x] `python3 .github/scripts/test-validate-release-candidate.py` still 48/48
- [x] `specsync change check` then `specsync change audit --strict`
- [x] Confirm no CRLF, reserved-name, path-separator, or `#[cfg(windows)]` code was removed
