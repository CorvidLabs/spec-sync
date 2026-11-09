# Testing

Fledge test task forwards filters. Run in named HC helpers with absolute tool identity, Rust 1.89, two build jobs and four test threads. Cases: positive current replacement/custody/retirement/eligible child rescope and closure; retired-only dependency refusal; unchanged obsolete criterion failure/canonical bytes; malformed or tampered anchors/packages; missing/stale/unreviewed replacement; exact owner/base and cycle refusals; repeated operation safety; legacy archives unchanged; old published 6.2 lifecycle consumers fail closed. Native check is not a product test. Full lane and trust results must remain distinct from human acceptance.

Requirement evidence: REQ-change-recovery-retirement-001, REQ-change-frozen-recovery-001, REQ-cmd-change-recovery-retirement-001, REQ-cli-args-recovery-retirement-001 are covered by src/change_tests/recovery_retirement.rs and CLI/integration tests.
