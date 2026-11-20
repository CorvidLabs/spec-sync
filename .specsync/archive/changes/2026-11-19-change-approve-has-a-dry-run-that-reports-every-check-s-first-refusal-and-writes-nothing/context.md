---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: context
---

# Context

Chough's `change ready` tries to say whether `specsync change approve` will accept a draft, and it keeps
its own copy of some of SpecSync's rules to do it. The copy lags. Against SpecSync 6.3.0 the Chough tester
found 25 kinds of draft where `change ready` says "Nothing is missing." and approve then refuses: a wrong
heading in a delta, an empty item or file, the same item twice, an ADDED requirement whose id exists or that
has no Acceptance Criteria, another module's requirement id, a path no module owns, a spec that does not
parse, and more (CorvidLabs/chough#311, #406). Issue #1091 asks SpecSync for the one source of truth: a dry
run of the approval that Chough can call.

Leif approved #1091 under the feature freeze on 2026-11-18, for the coming release. On depth he chose "First
problem from each check": run every check that can run on its own, report the first problem from each, and
say which checks were skipped. Listing every problem inside one check would mean reworking the shared
validators, and it is not in this release.

Leif approved this definition on 2026-11-19 at about 05:06Z in a Claude Code chat with his planning session
(hc t4c), relayed through the spec-sync lead agent (ss-lead, hc t58): "OK, with (a) fixed" for the intent
(CHANGE-15 to CHANGE-15.c) and "Approve" for the requirements, the criteria and four choices: which checks
are skipped, that the dry run takes no lock, that it refuses when an interrupted write is waiting, and that
the actor is its own check.

What a session picking this up needs to know:

- A refused `change approve` writes exactly one thing today: `.specsync/change.lock`, when it is absent.
  Taking the lock also recovers an interrupted lifecycle write, which is a write. So the dry run takes no
  lock and reads as `change status` does.
- The approval's checks live in `approve_definition_with_relay` (`src/change/sdd_version.rs`) and in the
  two functions that build the approval record (`src/change/validate_loaded_change.rs`). The dry run must
  walk the same list, not a copy of it.
- A real refusal prints `{"error": "<message>"}` with `--json` and `error: <message>` in text. The dry
  run prints its own document and never that one.
- REQ-change-163 and REQ-cmd-change-040 were free when this was written: 161 is taken by #1089, and 162, 164
  and 165 by branches from 2026-11-13; REQ-cmd-change-038 by #1089 and 039 by #1093.
