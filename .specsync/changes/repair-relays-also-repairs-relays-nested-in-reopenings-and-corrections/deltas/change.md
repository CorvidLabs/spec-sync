## MODIFIED

### REQUIREMENT REQ-change-relay-repair-001

`repair_relayed_decisions` SHALL repair every damaged `relayed_by` in an active change: in `approvals.json`, each approval and each reopening's `superseded_approval`; in `corrections.json`, each correction's `superseded_definition_approval` and `superseded_closing_approval`; and in `review-attempts.json` and `review.json`. Damaged means the reader refuses it: a shape it cannot read, or a `decided_by` that is not the record's own actor or reviewer. Each one SHALL be put back from the first copy of the same record that still exists and reads cleanly: the record at the same place in the file as committed at `HEAD`, then, for a review, its twin, because `review.json` and the last review attempt are copies of each other. A copy is the same record when every field other than `relayed_by` is equal. With no such copy, only that record's `relayed_by` SHALL be removed. No approval, reopening, correction or review attempt SHALL be removed and no verdict changed, so no blocking review is erased (REQ-change-046). For an active change, every refusal caused by a damaged relay SHALL name `specsync change repair-relays <id>`.

#### Acceptance Criteria

- Between `change review` and `change ship`, when neither review file is committed, a damaged relay in `review.json` is put back from the last review attempt, and one in the last attempt is put back from `review.json`, byte for byte; the change then ships with nothing committed in between.
- A damaged relay in a committed `approvals.json` is put back from `HEAD`, and the file's bytes equal the committed file.
- A damaged relay in a reopening's `superseded_approval`, or in an approval a correction embeds, is put back from the same place at `HEAD` when the committed file holds the same record, and is otherwise removed; the reopening or correction, and every other record in it, stays.
- A damaged relay with no surviving copy, such as an earlier review attempt or an approval that was never committed, is removed; every approval and attempt stays with its verdict, and the change can ship.
- A record whose relay reads cleanly is never changed, and with nothing damaged nothing is written.
- Files are written the way SpecSync writes them, all in one step under the project lock. A file that still does not read after its relays are repaired is refused and nothing is written.
- Because no digest and no gate reads a relay, a repair stales no approval, verification or review.
- An archived change is refused, writing nothing, naming `git checkout HEAD -- <archived package>`, because finalize commits the archive and `HEAD` holds every original. A damaged relay read from an archived package keeps naming `git checkout HEAD -- <file>`.
- A file that is not JSON is refused, writing nothing: the refusal names `git checkout HEAD -- <file>` when `HEAD` holds the file, and otherwise says that no copy of it exists.
