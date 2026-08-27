---
id: drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows
state: archived
type: operations
base_commit: d508f144a1d965b395abfe45f23c8b4e8978cd5f
---

# Drop the Windows binary from the 6.0 release matrix and stop claiming Windows support, while keeping every Windows-content correctness guarantee

## Intent

Drop the Windows binary from the 6.0 release matrix and stop claiming Windows support, while keeping every Windows-content correctness guarantee

## Affected Canonical Specs

- `github`
- `cli`
- `cmd_migrate`
- `change`
- `commands`
- `view`

## Acceptance Criteria

- The release and RC asset lanes build five artifacts, not six: no x86_64-pc-windows-msvc entry, no pwsh Compress-Archive packaging step, and no .zip glob remains in either lane. EXPECTED_ARTIFACT_ARCHIVES in validate-release-candidate.py no longer lists specsync-windows-x86_64.exe, so the exact-set artifact gate passes on the five archives actually produced instead of failing closed on a missing Windows directory. action.yml refuses a Windows runner with an actionable error naming WSL rather than 404ing on an asset that is no longer published. README, the site's Available Binaries table, the Multi-Platform Matrix example, the quickstart download note, and the adversarial-proof CI claim no longer state or imply that a Windows executable is shipped. Every correctness guarantee for Windows-authored content survives unchanged: parser CRLF normalization, strip_frontmatter, the .gitattributes eol=lf pins, is_reserved_module_name, validate_module_name, MAX_SLUG_BYTES, the portable_output_path and slash_normalized_relative_path helpers, and all cfg(windows) code and fixtures. Requirement wording that scoped those guarantees to 'every platform SpecSync ships a binary for' is rebound to the host platforms a repository may be checked out on, so narrowing the shipped set cannot narrow the guarantee. The Ubuntu/macOS/Windows release-candidate qualification lane is untouched.

## No-spec Rationale

Not applicable
