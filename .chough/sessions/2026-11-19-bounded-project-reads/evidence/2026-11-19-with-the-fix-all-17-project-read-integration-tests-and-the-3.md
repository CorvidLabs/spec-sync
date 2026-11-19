---
kind: evidence
id: 2026-11-19-with-the-fix-all-17-project-read-integration-tests-and-the-3
aliases: []
claim: "With the fix, all 17 project-read integration tests and the 3 new util unit tests pass, the source scan finds no direct project file read left, and the full verify lane passes: unit 3098 passed, 0 failed, 2 ignored; integration 665 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts."
result: passed
evidence_kind: product-test
method: "fledge lanes run verify with an isolated git config (10 steps, 19 min); cargo test --locked --test integration -- project_reads:: project_links:: (37 passed); rustc 1.89.0"
revision: 3a02a5d5903b08612c0dad8c44ecfdf28fe2b0ef
content: c61d228eac8868b38f9d758247a7fb5b936151d7e26b536900f8eb290e639d9b
environment: local-macos-arm64
criteria: [SETUP-7]
requirements: []
contracts: []
limitations:
  - "macOS arm64; the Linux container run follows after commit. No hosted CI: the private fork runs no Actions. 5 tests failed in the first full run and were fixed before this: two were defects in this change (Cargo's own config files outside the project were refused; hooks uninstall stopped failing on a linked-out CLAUDE.md, which REQ-hooks-006 requires), three expected the old message or read a --template directory, which is out of scope."
observed: 2026-11-19T15:13:01Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the fix, all 17 project-read integration tests and the 3 new util unit tests pass, the source scan finds no direct project file read left, and the full verify lane passes: unit 3098 passed, 0 failed, 2 ignored; integration 665 passed, 0 failed; fmt, clippy on all targets, the release build, the strict spec check at 62 specs and 100 percent coverage, and the gate scripts.**

Recorded as passed in local-macos-arm64 at revision 3a02a5d5903b, observed 2026-11-19T15:13:01Z.
