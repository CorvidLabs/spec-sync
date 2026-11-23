---
kind: session
id: 2026-11-11-ci-musl-apt-fallback
aliases: [ci-musl-apt-fallback]
objective: "The musl release-baseline CI jobs install musl-tools even when the runner's Ubuntu mirror stops resolving: apt retries, then falls back to other Ubuntu mirrors, and fails with ::error:: only when every mirror fails (run 43037386037)"
intent_scope: CI
modules: [github]
stage: accept
stage_status: archived
iteration: 1
native_change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
criteria: [CI-7, INSTALL-1.c]
closed: 2026-11-13
outcome: done
---
# The musl release-baseline CI jobs install musl-tools even when the runner's Ubuntu mirror stops resolving: apt retries, then falls back to other Ubuntu mirrors, and fails with ::error:: only when every mirror fails (run 43037386037)

## Goal

The musl release-baseline CI jobs install musl-tools even when the runner's Ubuntu mirror stops resolving: apt retries, then falls back to other Ubuntu mirrors, and fails with ::error:: only when every mirror fails (run 43037386037)

## Where it stands

## Decisions and why

- Authorization: Leif approved this fix in the coordinator chat on 2026-11-11 ('yes fix it!?') answering the coordinator's 'can I change that CI step to retry and fall back to a different server?', relayed by hc t370. Scope: the musl-tools install step of ci.yml linux-baseline, its stubbed test and its CI/fledge wiring; no other job and no required-checks change. Leif's standing bug-fix rule (2026-11-10) covers the agent-recorded approval and review; implementation acceptance is native review by claude-code-t384.
- Fallback mirror order: mirrors outside ubuntu.com first, the Canonical host (archive.ubuntu.com / ports.ubuntu.com) last. Rejected 'archive.ubuntu.com first' because run 43037386037 shows archive.ubuntu.com and security.ubuntu.com already in the x86 runner's mirror list and failing to resolve, and ports.ubuntu.com failing on arm64.
- release.yml and rc-assets.yml keep their bare apt-get musl install: outside the approved scope ('don't change any other job'); reported to t370 as a follow-up.
- Native definition approval: the classifier blocked the agent's approve, so Leif ran 'specsync change approve' himself (actor 0xLeif) on 2026-11-11, relayed by hc t370; t370 checked signature checks stay on and backup mirrors come before Canonical's.

## What was verified

## Bugs

None.

## Open threads

## Timeline

- 2026-11-11T16:01:31Z · advance · intent → design ·  ^t-160bf7
  - native_change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
  - revision: ad8544e9da79725f499b38ae4d4b3a052be5f807
- 2026-11-11T16:01:31Z · skip · design → plan · CI-only change: no visuals or interactions ^t-36ace0
  - native_change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
  - revision: ad8544e9da79725f499b38ae4d4b3a052be5f807
- 2026-11-11T16:04:14Z · advance · plan → build · native:approved ^t-b6acdf
  - native_change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
  - gate: native:approved
  - revision: ad8544e9da79725f499b38ae4d4b3a052be5f807
- 2026-11-11T16:04:16Z · advance · build → test · native:approved ^t-06bba0
  - native_change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
  - gate: native:approved
  - revision: ad8544e9da79725f499b38ae4d4b3a052be5f807
- 2026-11-11T16:04:18Z · advance · test → accept · native:approved ^t-9b3e0a
  - native_change: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
  - gate: native:approved
  - revision: ad8544e9da79725f499b38ae4d4b3a052be5f807

Closed 2026-11-13: done.

## Receipts

```yaml
receipts: []
```
