---
kind: session
id: 2026-11-24-642-site-pages-read-as-published
aliases: [642-site-pages-read-as-published]
objective: "The quickstart and two example pages read as published for 6.4.2: the three 'until then, pin 6.4.1' sentences the preparation added go"
modules: [github]
stage: done
stage_status: archived
iteration: 1
native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
criteria: [CI-1]
observations: [2026-11-24-each-of-the-three-pages-quickstart-md-examples-polyglot-mdx]
summary:
  - SpecSync 6.4.2 is published, so three site pages no longer tell readers to pin 6.4.1 until then.
closed: 2026-11-24
outcome: done
---
# The quickstart and two example pages read as published for 6.4.2: the three 'until then, pin 6.4.1' sentences the preparation added go

## Goal

The quickstart and two example pages read as published for 6.4.2: the three 'until then, pin 6.4.1' sentences the preparation added go

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2": "I approve all three" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The second was yes or no to fixing the three stale 'until then, pin 6.4.1' sentences on the quickstart page and the two example pages, as a second three-line change in the same pull request. The message before it had shown the three edits in a table and said a yes would be recorded as both the scope approval and the implementation review of that change.); recorded by claude-code-t14v; a relayed claim, not authenticated; ran specsync change approve the-quickstart-and-two-example-pages-read-as-published-for-6-4-2 as 0xLeif
- 0xLeif decided "Implementation review of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2": "I approve all three" (Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The second was yes or no to fixing the three stale 'until then, pin 6.4.1' sentences on the quickstart page and the two example pages, as a second three-line change in the same pull request. The message before it had shown the three edits in a table and said a yes would be recorded as both the scope approval and the implementation review of that change; the edits made are exactly those.); recorded by claude-code-t14v; a relayed claim, not authenticated; the scoped review of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2 was already current, so no review ran

## What was verified

- Each of the three pages (quickstart.md, examples/polyglot.mdx, examples/ci-gate.mdx) is byte for byte its text before the 6.4.2 preparation with only that preparation's 6.4.2 pins kept; no page under site/src or docs still says 'until then' or 'assets are published'. The scoped check passes. ^2026-11-24-each-of-the-three-pages-quickstart-md-examples-polyglot-mdx

## Bugs

None.

## Open threads

## Timeline

- 2026-11-24T22:46:39Z · start · intent → intent · The quickstart and two example pages read as published for 6.4.2: the three 'until then, pin 6.4.1' sentences the preparation added go ^t-e28f58
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:05Z · advance · intent → design ·  ^t-74cad9
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:05Z · skip · design → plan · Three sentences of documentation. No visuals or interactions. ^t-b848c0
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:05Z · gate-opened · plan → plan · Scope approval of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2 ^t-eb8467
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - holds: plan
  - drives: approve
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:06Z · direct-native-write · plan → plan · the-quickstart-and-two-example-pages-read-as-published-for-6-4-2's lifecycle records changed outside Chough: state.json changed, approvals.json changed; state draft → approved ^t-d2efa9
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - direct_write:
    - change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
    - files:
      - state.json: changed
      - approvals.json: changed
    - before_state: draft
    - after_state: approved
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:06Z · gate-closed · plan → plan · Scope approval of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2 ^t-57cf72
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - relayed_by:
    - decided_by: 0xLeif
    - words: I approve all three
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The second was yes or no to fixing the three stale 'until then, pin 6.4.1' sentences on the quickstart page and the two example pages, as a second three-line change in the same pull request. The message before it had shown the three edits in a table and said a yes would be recorded as both the scope approval and the implementation review of that change.
    - recorded_by: claude-code-t14v
    - native: specsync change approve the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
    - actor: 0xLeif
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:08Z · advance · plan → build · native:approved ^t-ead329
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - gate: native:approved
  - revision: 9caed41498939ad7fdf6d74c2dd1c4eab3413bf0
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:47Z · advance · build → test · native:verifying ^t-384920
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - gate: native:verifying
  - revision: eb3a9ebcc6853cbc5070a5b22320eee5645b864f
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:50Z · advance · test → accept · native:verifying ^t-ae9733
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - gate: native:verifying
  - revision: eb3a9ebcc6853cbc5070a5b22320eee5645b864f
  - by: claude-code (hc t14v)
- 2026-11-24T22:47:50Z · gate-opened · accept → accept · Implementation review of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2 ^t-28050c
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: eb3a9ebcc6853cbc5070a5b22320eee5645b864f
  - holds: accept
  - drives: review
  - by: claude-code (hc t14v)
- 2026-11-24T22:51:05Z · direct-native-write · accept → accept · the-quickstart-and-two-example-pages-read-as-published-for-6-4-2's lifecycle records changed outside Chough: review.json added, review-attempts.json added; state stays verifying ^t-cf3a11
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: b5c2d3bad207cb52115f1d9500cd600cebbd920b
  - direct_write:
    - change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
    - files:
      - review.json: added
      - review-attempts.json: added
    - before_state: verifying
    - after_state: verifying
  - by: claude-code (hc t14v)
- 2026-11-24T22:51:05Z · gate-closed · accept → accept · Implementation review of the-quickstart-and-two-example-pages-read-as-published-for-6-4-2 ^t-9191b0
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - revision: b5c2d3bad207cb52115f1d9500cd600cebbd920b
  - relayed_by:
    - decided_by: 0xLeif
    - words: I approve all three
    - where: Claude Code release-lead chat, 2026-11-24 UTC, typed by him to the agent recording it. It answered a message that asked three numbered questions. The second was yes or no to fixing the three stale 'until then, pin 6.4.1' sentences on the quickstart page and the two example pages, as a second three-line change in the same pull request. The message before it had shown the three edits in a table and said a yes would be recorded as both the scope approval and the implementation review of that change; the edits made are exactly those.
    - recorded_by: claude-code-t14v
  - by: claude-code (hc t14v)
- 2026-11-24T22:51:07Z · advance · accept → done · native:archived ^t-792831
  - native_change: the-quickstart-and-two-example-pages-read-as-published-for-6-4-2
  - gate: native:archived
  - revision: b5c2d3bad207cb52115f1d9500cd600cebbd920b
  - by: claude-code (hc t14v)

Closed 2026-11-24: done.

## Receipts

```yaml
receipts:
  - id: 2026-11-24-each-of-the-three-pages-quickstart-md-examples-polyglot-mdx
    claim: Each of the three pages (quickstart.md, examples/polyglot.mdx, examples/ci-gate.mdx) is byte for byte its text before the 6.4.2 preparation with only that preparation's 6.4.2 pins kept; no page under site/src or docs still says 'until then' or 'assets are published'. The scoped check passes.
    result: passed
    revision: eb3a9ebcc6853cbc5070a5b22320eee5645b864f
    environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone
    content: 4ae26f9f7bb678217431e3b1b88e2af5b9ce523c84242a71c8da1ddcda2c6100
    criteria: CI-1
    observed: 2026-11-24T22:47:47Z
    actor: claude-code (hc t14v)
```
