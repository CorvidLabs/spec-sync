---
change: other-write-errors-still-panic-and-test-output-stays-captured
artifact: design
---

# Design

The closed-pipe handling in `write_output_stream` now delegates to `std::print!`/`std::eprint!` when
`cfg!(test)`, so unit tests keep libtest's output capture. The integration test points stdout at
`/dev/full` (Linux) and asserts exit 1 with the panic reported as a bug.
