---
kind: evidence
id: 2026-11-19-with-the-fix-at-a33dcc6f-all-19-project-link-integration-tests
aliases: []
claim: "With the fix at a33dcc6f, all 19 project-link integration tests pass and the unit tests of every touched module pass (239). The same suite passes on Linux arm64 in a local container as a normal user: unit 3099 passed, integration 650 passed plus one test that needs jq and passes once jq is installed."
result: passed
evidence_kind: product-test
method: "macOS: cargo test --locked --test integration -- project_links:: (19 passed); cargo test --locked --bin specsync on the touched modules (239 passed); cargo clippy --all-targets -- -D warnings; specsync check --strict --require-coverage 100 --force (62 specs). Linux: rust:1.89-bookworm container, cargo clippy and cargo test --locked --no-fail-fast on a clean copy of a33dcc6f; rustc 1.89.0"
revision: a33dcc6ffd6210e4998c376edab1d331083bcd7f
content: 832a247c578e6ffff491fae9f2cbe21f1e457b4c792e5da8b4254ec7becd3b64
environment: local-macos-arm64
criteria: [SETUP-6]
requirements: []
contracts: []
limitations:
  - "The full macOS verify lane had not finished when this was recorded. Linux was arm64 on Debian 12, not x86_64 and not the Ubuntu image CI uses. No hosted CI: the private fork runs no Actions. Not driven: a live resolve --remote, and the wizard's prompts. Requirement ids not linked with --requirement for modules with an underscore in their name."
observed: 2026-11-19T05:30:29Z
actor:
  - agent: claude-code
    terminal: t5r
---
**With the fix at a33dcc6f, all 19 project-link integration tests pass and the unit tests of every touched module pass (239). The same suite passes on Linux arm64 in a local container as a normal user: unit 3099 passed, integration 650 passed plus one test that needs jq and passes once jq is installed.**

Recorded as passed in local-macos-arm64 at revision a33dcc6ffd62, observed 2026-11-19T05:30:29Z.
