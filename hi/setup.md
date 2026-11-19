---
hi: 1
families: [SETUP]
---

# Getting started

## Intent

Adopting this on a real project should be a small, reversible decision, not a migration. One command lays out sensible defaults and asks nothing, the heavier change workflow stays off until somebody deliberately turns it on, and configuration is one readable file where every setting already has a default worth keeping. Work that already exists somewhere else — an issue backlog, a folder of documents, another spec-driven tool's output — should be something I can bring in rather than retype.

SETUP-6 comes from a private security sweep done with GHSA-7q5v-8jcc-rx54: install, scaffold, import, lifecycle and migrate commands wrote through symbolic links a repository can commit. Leif decided "Fold into the same private fix" and, asked whether a link that stays inside the repository may still be written through, "Allow links that stay inside" (Claude Code chat with his planning session, hc t4c, 2026-11-19 at about 04:53Z). His planning session put that answer into the criterion's wording and relayed it as his. Recorded by the security builder agent (hc t5r).

SETUP-7 comes from a tester's finding handled with the same advisory: SpecSync read `.specsync/sdd.json` and `.specsync/change-transaction.json` through symbolic links with no size bound, so a link to `/dev/zero` exhausted memory and a FIFO made a command wait forever. Leif decided "Yes, a 3rd change" and approved its scope, then approved this criterion as written ("OK as written", Claude Code chat with his planning session, hc t4c, 2026-11-19 at about 14:29Z), relayed by t4c. Recorded by the security builder agent (hc t5r).

## Criteria

- **SETUP-1**  One command sets a project up with defaults that work and nothing I have to answer.
  - **SETUP-1.a**  Setting up does not switch on the change workflow; that stays a separate, deliberate decision.
    - **SETUP-1.a.1**  Every current contract agrees that setting up a git repository leaves the change workflow off, the same as any other directory.
    - **SETUP-1.a.2**  If a current contract goes back to promising the change workflow is on after setup, a test tells me.
  - **SETUP-1.b**  A half-finished setup can be repaired without touching the settings I already wrote.
- **SETUP-2**  Configuration is one readable file.
  - **SETUP-2.a**  Every setting has a default worth keeping, so a project that configures nothing still works.
  - **SETUP-2.b**  I can say where my contracts, sources and schema actually live.
  - **SETUP-2.c**  A configuration file it cannot parse stops the run instead of silently falling back to defaults.
  - **SETUP-2.d**  A project can decide whether a failing check blocks the work or only reports it.
- **SETUP-3**  Upgrading from an older layout is a guided migration rather than a hand edit.
  - **SETUP-3.a**  A migration backs up what it is about to change.
  - **SETUP-3.b**  I can preview a migration before it writes anything.
- **SETUP-4**  Switching the change workflow on is one command that leaves my existing settings and history alone.
- **SETUP-5**  I can bring in work that already exists elsewhere — an issue tracker, a wiki page, a folder of documents — as a starting draft.
  - **SETUP-5.a**  An imported draft is honestly marked incomplete rather than presented as a passing contract.
  - **SETUP-5.b**  Work already laid out by another spec-driven tool can be adopted rather than retyped.
  - **SETUP-5.c**  I can bring in a whole backlog in one go rather than an item at a time.
- **SETUP-6**  A command I run in a repository somebody else wrote never writes outside it through a link it found there.
- **SETUP-7**  A command I run in a repository somebody else wrote never reads outside it through a link it found there, and no file there can make it hang or run out of memory.
