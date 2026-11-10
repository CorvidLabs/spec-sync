---
change: change-review-refuses-to-pass-a-change-finalize-would-refuse-on-ownership
artifact: testing
---

# Testing

Unit tests in `src/commands/change.rs` and the integration test
`ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`
drive approve, check, review and ship on temp git projects.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-008 | integration `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships`; unit `an_archive_candidate_with_unowned_production_source_is_not_told_to_ship`, `the_ownership_recovery_names_a_repair_for_every_unowned_input`, `reserved_delivery_paths_are_not_ownership_blockers` |
