---
change: fix-the-four-pr-1086-review-findings-on-history-fragments
artifact: design
---

# Design

- **Quote.** `history_fragment_content` writes `change: "<id>"` with `\"` and `\\` escapes; `validate_change_id` already refuses control characters and `\`. `yaml_scalar` reads double-quoted (only `\"` and `\\` escapes), single-quoted (`''`) or plain values.
- **Entry types.** `validation_input_digest_for_build` frames `<path>=<file|directory|symlink|other|missing>` for `history/` and each entry, sorted, through `symlink_metadata`, beside the paths whose bytes it already hashes.
- **Change Log items.** `refuse_change_log_delta_items` runs beside `validate_delta_files` in approve, materialize, verify and accept. It is not in the project audit, so an archived change that once carried such an item is not re-judged.
- **One matcher.** `changelog_records_change` now requires that the character before `<id>:` is not alphanumeric, `-`, `_` or `.`; `scan_history_fragments` uses it for the double record, and the narrower `changelog_row_names_change` is gone.
