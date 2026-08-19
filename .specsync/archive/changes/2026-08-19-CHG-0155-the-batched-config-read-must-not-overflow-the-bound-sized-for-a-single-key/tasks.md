# Tasks

- [x] Reproduce: measure the batched query's output at two scopes (144 bytes vs a 128 bound)
- [x] Confirm the sibling fsmonitor read uses 16 KiB for the same shape
- [x] Write the two-scope test; confirm it fails on 76ef32b1 with the bounds error
- [x] Raise the bound; confirm the test passes and the other four config tests are unchanged
- [x] Assert equivalence against `git config --get` rather than a guess about precedence
