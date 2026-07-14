---
change: CHG-0026-keep-lifecycle-recursion-detection-private-while-preserving-deterministic-nested
artifact: testing
---

# Testing

- `cargo test indirect_recursive_lifecycle_check_fails_once_with_context`
- `cargo test indirect_recursive_lifecycle_subcommands_fail_once_with_context`
- `fledge run fmt`
- `fledge run check-types`
- Configured lifecycle verification: `cargo test`
