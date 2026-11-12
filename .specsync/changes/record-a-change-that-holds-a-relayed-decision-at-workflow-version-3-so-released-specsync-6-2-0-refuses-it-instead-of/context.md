---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: context
---

# Context

#1064, a follow-up to #1043 and #1044. `relayed_by` (#633, #900) and `cancelled_review_attempts` (#995) landed on main after 6.2.0. Released 6.2.0 deserializes `approvals.json` and the review ledgers through typed structs that ignore unknown fields, then re-serializes them on every lifecycle rewrite. So the first time 6.2.0 writes to a change on which a newer binary recorded a relayed decision, it drops `relayed_by` and exits 0. #1044 made newer binaries keep unknown members, but 6.2.0 is already released. The only lever is something 6.2.0 already refuses: `load_change` rejects a `state.json` whose `workflow_version` is not 1 or 2, with "was written by a newer SpecSync (workflow version N); upgrade specsync to read it".

Leif chose option (a), his word "A", relayed by the coordinator agent t370 on 2026-11-11: 6.3.0 must stop released 6.2.0 from rewriting a change record that carries a relayed decision, rather than let it silently drop `relayed_by`. t370 checked that REQ-change-158's wording matches that choice before approval.

What a session picking this up needs to know:

- The gate covers the whole repository, not one change. I measured this with the real 6.2.0 binary on a scratch repository holding a hand-raised version-3 record (research.md). 6.2.0 lists every change, so one active version-3 change also makes it refuse `approve` on any other change, plus `change list` and `change audit`. One archived version-3 change makes it refuse `change new` and `change audit` for good. `check` (including `--strict --require-coverage 100 --force`), `lifecycle enforce --all` and `comment` still pass. So the GitHub Action and Trust v1.2.1, which runs only `specsync check`, keep working when pinned to 6.2.0. In plain terms: once a repository records one relayed decision, every machine that runs change commands there needs SpecSync 6.3.0 or later. The PR body and the 6.3.0 release notes say so (t370's request).
- Chough 0.6.0 passes relays to `change review` from `chough change ship`, so CorvidLabs repositories will reach this state quickly.
- Workflow-v1 changes are not gated. The `--portable-5-0-1` pair requires v1 and exists so old binaries can read it; approve and review print a note. t370 chose this over refusing v1 relays (coordinator, 2026-11-11), which keeps REQ-change-relayed-decision-001 intact. The PR body and the release notes say: relays on legacy workflow-v1 (portable 5.0.1) changes are not gated, and approve and review say so.
- Not retroactive: 7 archived changes in this repository and 1 in CorvidLabs/site already hold relays at version 2. Archives are not edited.
- The issue's part (b), a latent array-length gap in `collect_unknown_members`, is out of scope.
