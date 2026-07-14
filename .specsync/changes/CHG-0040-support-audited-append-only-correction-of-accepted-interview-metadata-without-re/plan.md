---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: plan
---

# Plan

1. Add typed correction fields, versioned audit records, effective-definition projection, and
   domain-separated portable digest-chain validation in `src/change.rs`.
2. Implement an atomic accepted-to-verifying correction transition that snapshots prior gate
   evidence, adds required artifacts monotonically, and preserves canonical application state.
3. Route approval, verification, acceptance, strict checking, summaries, and inspection through the
   effective corrected definition while retaining legacy behavior when no ledger exists.
4. Add the Clap grammar and thin text/JSON command adapter for `change correct`.
5. Add focused unit and integration regressions for state safety, malformed history, portability,
   squash integration, repeated corrections, non-replay, and deterministic output.
6. Apply the `change`, `cli_args`, and `cmd_change` semantic deltas; update companion specs, public
   workflow/CLI docs, and the unreleased changelog.
7. Run formatting, the complete Rust suite, strict spec validation, documentation build, audit, and
   the repository trust gate before presenting closing approval evidence.
