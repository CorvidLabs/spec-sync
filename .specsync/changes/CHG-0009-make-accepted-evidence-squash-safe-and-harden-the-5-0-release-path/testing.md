---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: testing
---

# Testing

- Covers `REQ-change-016`.
- Reproduce a feature branch acceptance followed by a squash merge onto the remote default branch.
- Prove unchanged accepted evidence passes and archives.
- Prove an unintegrated branch and changed scoped input both fail.
- Run `cargo test`, strict SpecSync checks, release packaging dry-run, site checks/build, and clean Action consumers.

## Results

- Rust: 1,514 unit and 187 integration tests passed.
- Formatting and Clippy with warnings denied passed.
- Strict lifecycle/spec validation passed: 62 specs, zero warnings/errors, 100% file and LOC coverage.
- The merged-tree archive migration succeeded for CHG-0001, 0002, 0004, 0006, 0007, and 0008.

## Invalidated Result

The first PR run passed every gate except Windows. Git-for-Windows converted temporary fixture files from LF to CRLF
after a branch switch, invalidating the fixture's byte-exact contract before the squash assertion. The prior closing
acceptance and verification evidence were removed, CHG-0009 returned to `implementing`, and these results must be
replaced only after the complete corrected matrix passes.

## Corrected Local Evidence

- 81 focused lifecycle tests pass, including both squash scenarios.
- NUL-boundary collisions, arbitrary binary changes, LF/CRLF distinctions, executable mode, symlink kind/target, and
  non-UTF-8 path rejection have dedicated regressions.
- Rust formatting, Clippy with warnings denied, and diff whitespace validation pass for the digest implementation.
- The crate manifest allowlist produces 113 package entries and excludes site, extension, specs, workflows, tests,
  agent instructions, and local dependency trees.
- The complete corrected Rust suite passes: 1,526 unit tests and 187 integration tests, zero failures.
- Effective lifecycle validation passes for both active changes with zero errors or warnings.
- Dependency validation passes across 62 modules and 215 edges with zero cycles, missing dependencies, undeclared
  imports, errors, or warnings.
- Documentation tests (23), Astro diagnostics/build (38 pages), VS Code compilation/package, and release workflow
  `actionlint` all pass locally.
- RustSec refresh is pending CI because the sandbox could not obtain its external advisory-database lock. The crate
  package allowlist is locally validated; crates.io dry-run publication remains a network-backed CI/release gate.
