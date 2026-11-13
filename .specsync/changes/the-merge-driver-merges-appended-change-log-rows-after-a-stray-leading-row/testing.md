---
change: the-merge-driver-merges-appended-change-log-rows-after-a-stray-leading-row
artifact: testing
---

# Testing

- Unit `two_appended_rows_merge_after_a_stray_leading_change_log_row` in `src/merge_driver.rs`,
  in both merge orders (fails on main: the merge conflicts).
- The existing merge-driver tests still pass.
