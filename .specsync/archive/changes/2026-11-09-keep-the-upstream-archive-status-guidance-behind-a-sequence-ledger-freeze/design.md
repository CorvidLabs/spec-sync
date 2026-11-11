---
change: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
artifact: design
---

# Design

One priority order for every surface: sequence-ledger freeze, then the verification_ignored_paths refusal, then the upstream archive, then state-local steps.

- Status summary (`src/change/historical_definition_digest_matches.rs`): use `upstream_archive_handoff` only when neither the freeze nor the verification_ignored refusal applies; otherwise `classify_handoff` already reports them.
- Text next action (`text_mode_next_action`, `src/commands/change.rs`, also ship-status's lifecycle next): return the freeze remediation before the upstream archive.
- `handoff_summary` already returns the freeze before checking the upstream archive.
