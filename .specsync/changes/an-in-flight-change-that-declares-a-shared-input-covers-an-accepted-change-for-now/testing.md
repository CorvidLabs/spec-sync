---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now
artifact: testing
---

# Testing

src/change_tests/provisional_coverage.rs:

- `an_in_flight_change_that_declares_the_input_covers_it_for_now`: a legacy accepted change, an
  approved later change declaring `auth` and `src/auth.rs`, and a committed edit of the file;
  the evidence reads `provisionally-covered` naming the later change, audit has no error and a
  warning naming both, and `archive_change` succeeds with `coverage.json` naming the file and the
  covering change. Red on main: the evidence is stale.
- `with_no_change_declaring_the_input_it_is_stale_as_before`: the same edit with no later change
  is stale and fails audit.
- Full unit suite and the change, doctor, archived-upstream and close integration tests; fmt and
  clippy clean.
