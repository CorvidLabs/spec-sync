---
change: finalize-validates-only-the-change-it-archives
artifact: research
---

# Research

On the #842 base, `cold_finalize_git_processes_do_not_grow_with_other_archives` measured 794 Git processes for one finalize with no persistent read cache and one other archive, and 1,123 with four more (329 more, about 82 per archive). Every one of `list_all_changes_checked`'s callers on the finalize path is in `archive_change_with_options`: the `simulated` roster for the post-move preflight, and `ensure_closing_approval_valid`. `accept_change_with_gate` loads only active records, and `ensure_dependencies_satisfied` loads each dependency through `load_change`, which validates that record's history. Another archive's evidence reaches a verdict about the target only as a semantic successor (`successor_covers_input`) or a sequence owner (`later_sequence_owner_covers_historical_input`), and both authenticate it through `AcceptedInputValidationMemo::authenticate`. `change audit` today ignores `--strict` and loads archives only as successor candidates, so it does not walk archive histories at all.
