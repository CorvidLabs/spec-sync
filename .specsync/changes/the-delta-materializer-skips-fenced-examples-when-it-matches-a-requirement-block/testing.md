---
change: the-delta-materializer-skips-fenced-examples-when-it-matches-a-requirement-block
artifact: testing
---

# Testing

- Unit `a_modified_delta_skips_a_fenced_example_of_the_same_id`: a MODIFIED delta replaces the
  real block and keeps the fenced example above it byte for byte (fails on main: the example is
  replaced).
- Unit `a_fenced_heading_inside_the_block_does_not_end_it`: the whole real block, its fenced
  example included, is replaced (fails on main: the block ends at the fenced heading).
- The titled-header (#1024), stale-MODIFIED and send-back tests still pass.
