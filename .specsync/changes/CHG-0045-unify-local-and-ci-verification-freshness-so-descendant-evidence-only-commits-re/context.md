---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: context
---

# Context

SpecSync currently evaluates a verifying workspace with two different commit-freshness rules. `change check` accepts an ancestor verification commit only when CI environment variables are present, while local checks require the verification commit to equal `HEAD`. `summarize_change` always requires equality. A supported `change verify` necessarily writes lifecycle evidence after testing the current commit; committing that evidence therefore makes a truthful local verification stale while hosted CI may still accept it.

The split is observable in the SpecSync 5.1 release workspace: CHG42, CHG43, and CHG44 each passed their configured native verification, but committing the generated evidence caused local strict checking to reject it. Treating `CI=true` as a local workaround would hide a contract inconsistency rather than prove portable behavior.

Freshness must instead depend on one environment-independent predicate. The verification commit must be `HEAD` or an ancestor of `HEAD`; its approved definition digest and project-input digest must still match; and every parent edge of every intervening commit may change only the three files produced by supported verification persistence: `state.json`, `verification.json`, and `verification-attempts.json` below a canonical `.specsync/changes/<change-id>/` directory. Source, tests, configuration, canonical specs, policy, archive records, approvals, tasks, sequence state, hashes, locks, or any other paths must invalidate the evidence. A source change followed by a revert must remain stale because the history itself crossed a governed-input boundary. Missing, divergent, and ambiguous history fails closed.
