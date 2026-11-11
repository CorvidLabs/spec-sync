---
change: other-write-errors-still-panic-and-test-output-stays-captured
artifact: testing
---

# Testing

Integration `a_write_error_other_than_a_closed_pipe_still_panics` (Linux only; CI runs it). A probe
unit test that prints is captured again under `cargo test` (checked by hand). The three
`closed_stdout` tests still pass.
