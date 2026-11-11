---
change: every-actor-accepting-verb-normalizes-identity-the-same-way
artifact: context
---

# Context






Round-2 review of #426 (PR #1026): resolve_actor covers approve/review/accept, but send-back
(src/change/send_back.rs:278), reopen (sdd_version.rs:4057), correct
(historical_definition_digest_matches.rs:940) and correct-owner (sdd_version.rs:4610) only
trimmed and checked empty — a newline actor still entered the append-only ledger.
