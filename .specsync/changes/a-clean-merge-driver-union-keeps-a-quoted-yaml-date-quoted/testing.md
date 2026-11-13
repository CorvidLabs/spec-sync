---
change: a-clean-merge-driver-union-keeps-a-quoted-yaml-date-quoted
artifact: testing
---

# Testing

- Unit `quoted_yaml_timestamps_stay_strings_after_a_union` in `src/merge_driver.rs`: a quoted
  `2026-11-07` stays quoted after a clean union in both merge orders (fails on main: it comes out
  unquoted); a full timestamp stays quoted; `docs/2026-11-07.md` stays plain.
- The existing #872 typed-item and merge-driver tests still pass.
