---
kind: evidence
id: 2026-11-19-1091-verify-lane-at-c4dba3f6
aliases: []
claim: "At c4dba3f6 (#1091 on main 3bebe8b0), fledge lanes run verify passed 10/10 steps in 10m37s: fmt, lint and check-types, cargo test (unit 3083 passed, 2 ignored; integration 635 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. It includes the 8 tests of tests/integration/approve_dry_run.rs (each faulty draft from the Chough tester's list gets one message from the dry run and from the real approve, with not one file changed; a clean draft; two problems in one run; a missing change; a sent-back change; the relay flags; an interrupted write), the 5 unit tests for the in-memory binding and raise and the skip rules, and the flag's parsing test. The lane's processes ran with a neutral global git config (#1092)."
result: passed
evidence_kind: product-test
revision: 22bbf2888974050b186d5d7f49c149a0513573d6
content: b69dde44c65a8f48f3f03a134f096808f258013bc8a9fe5d3d41771e073c47df
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
fingerprint: sha256:13dcec7a1639d49415f1ec769cb9ae33df536808a6aec332672e4b64a80a0f1d
external: ext-ddf04a
criteria: [CHANGE-15, CHANGE-15.a, CHANGE-15.b, CHANGE-15.c]
requirements: [REQ-change-163, REQ-cmd-change-040, REQ-cli-args-027]
contracts:
  - specs/change/change.spec.md
  - specs/cmd_change/cmd_change.spec.md
  - specs/cli_args/cli_args.spec.md
limitations: []
observed: 2026-11-19T06:37:37Z
actor:
  - agent: claude-code
    terminal: t59
---
**At c4dba3f6 (#1091 on main 3bebe8b0), fledge lanes run verify passed 10/10 steps in 10m37s: fmt, lint and check-types, cargo test (unit 3083 passed, 2 ignored; integration 635 passed), release build, strict spec check (62 specs, 100% file and LOC coverage) and the CI script tests. It includes the 8 tests of tests/integration/approve_dry_run.rs (each faulty draft from the Chough tester's list gets one message from the dry run and from the real approve, with not one file changed; a clean draft; two problems in one run; a missing change; a sent-back change; the relay flags; an interrupted write), the 5 unit tests for the in-memory binding and raise and the skip rules, and the flag's parsing test. The lane's processes ran with a neutral global git config (#1092).**

Recorded as passed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone at revision 22bbf2888974, observed 2026-11-19T06:37:37Z.
