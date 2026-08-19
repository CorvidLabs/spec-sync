# Tasks

- [x] Measure the spawn cost (~15 ms) and the spawn count (15,359 → 3,842)
- [x] Verify `--get-regexp -z` equivalence against real git across six cases
- [x] Replace the four reads with one snapshot; derive all four values from it
- [x] Keep the malformed-config path failing loudly
- [x] Exclude `core.fsmonitor` and record why
- [x] Delete the wrapper left dead by the change
- [x] Add an equivalence test and a malformed-config control; confirm both pass on BOTH binaries
- [x] Clean sequential A/B on an idle machine
