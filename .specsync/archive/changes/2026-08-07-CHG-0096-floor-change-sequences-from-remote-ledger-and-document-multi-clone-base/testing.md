---
change: CHG-0096-floor-change-sequences-from-remote-ledger-and-document-multi-clone-base
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|---------|
| REQ-change-055 | maximum_observed_sequence_floors_on_remote_ledger; sequence_base_env_raises_high_water |

`cargo test floors_on_remote sequence_base_env`
