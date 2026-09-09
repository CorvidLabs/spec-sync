---
change: correct-the-6-0-init-version-stamp-quickstart-example-and-release-notes
artifact: testing
---

# Testing

## Automated

- `tests/integration/quickstart.rs` (new, registered in `tests/integration.rs`):
  - `quickstart_example_passes_strict_check_with_full_coverage_from_its_own_root` copies
    `examples/quickstart/` to a temp dir and asserts `check --strict --require-coverage 100`
    succeeds, prints `1/1 exports documented`, `0 warning(s), 0 failed`, and
    `File coverage: 1/1 (100%)`.
  - `quickstart_example_reports_the_readme_undocumented_export` appends the README's `farewell`
    function and asserts plain `check` succeeds while printing `Undocumented export 'farewell'`,
    and `check --strict` fails on it.
  - Both tests fail against the previous spec (0/1 coverage, export validation skipped).
- `commands::init::tests::write_current_layout_creates_full_structure` compares the stamp to
  `PROJECT_VERSION = SDD_VERSION`, so it pins `6.0.0` without a text change.
- `every_integration_test_file_is_registered` guards the new module registration.

## Manual (release binary, this review)

- Fresh `init` in an empty git repository → `.specsync/version` reads `6.0.0`; `check`,
  `change adopt`, `change audit`, `generate` exit 0.
- `examples/quickstart`: `check` from its own root shows the output now printed in its README;
  adding `farewell` yields the warning; `--strict` exits 1.
- Full suite before the change: 2465 unit + 410 integration green (two unit tests need a
  non-root user, two integration tests need `USER` set; both hold in CI).
