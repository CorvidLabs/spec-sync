---
change: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
artifact: context
---

# Context

Issue #608 (P1, G-608 in the 2026-11-06 triage): CI's `test` job ran `cargo clippy -- -D warnings` (`ci.yml` line 241), and so did the fledge `lint` task (`fledge.toml` line 14). Without `--all-targets`, clippy compiles only the library and binary targets, so it never lints a `#[cfg(test)]` module or `tests/`. A green lint run meant that the shipped code lints clean, but it read as the code linting clean.

Measured on main ab688246 with the pinned toolchain (1.89): `cargo clippy --all-targets -- -D warnings` exits 101 with 36 errors. 35 are in the binary's test target and 1 is in `tests/integration`, across 15 files. #608 counted 20 at ecff965e, so test code kept accumulating lints nobody saw. All 36 are in test-only code: plain `cargo clippy -- -D warnings` passes.

HI: CI-8 is new, in Leif's words: "When the pipeline says the code lints clean, that includes the tests." (Herdr orc tab, 2026-11-06, choice "A", relayed by orc t18). REQ-github-028 was claimed with the orc.

Ruled out: `allow`/`expect` attributes, lowering a lint group, or passing `-A`/`--cap-lints`. Each would turn the gate green by looking away, which is the defect class #608 names.
