# Plan

Raise the bound to `16 * 1024`, matching the sibling `core.fsmonitor` read at
`src/change.rs:12053`, which faces the same "one query, unknown number of records" shape.

The guard itself stays. This is not removing a bound — it is sizing the bound for the response
the call can actually receive rather than for the response the call it replaced received.

## The test that was missing

A fixture with all four keys set in **two** scopes: an `include.path` file plus local
overrides. Measured at 144 bytes, so it fails on `76ef32b1` with the exact bounds error and
passes with the fix.

Its assertion compares each derived value against `git config --get <key>` **run in the same
fixture**, rather than hardcoding which scope wins. The first version of this test asserted that
local overrides the include; git resolved the other way, so the test failed while the code was
correct. Asserting the property instead of the guess is what makes it durable.
