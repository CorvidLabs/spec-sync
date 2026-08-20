# Plan

1. `located_change_ordinal` — distinguish "claims no ordinal" from "claims one badly".
2. `LocatedChangeSequence.sequence` becomes `Option<u64>`; numeric accounting skips `None`.
3. Explicit duplicate-identity gate in `validate_change_sequences`, keyed on `record.id`.
4. Allocator mints a slug; a repeated description is refused by name.
5. Stop force-appending `SEQUENCE_PATH` into new changes' `affected_paths`.
6. Keep and comment the ~400 lines of history-reading machinery the 120 archives need.
7. Measure: full suite, per-risk-class corpus sample, and a slug-only lifecycle end to end.
