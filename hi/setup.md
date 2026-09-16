---
hi: 1
families: [SETUP]
---

# Getting started

## Intent

Adopting this on a real project should be a small, reversible decision, not a migration. One command lays out sensible defaults and asks nothing, the heavier change workflow stays off until somebody deliberately turns it on, and configuration is one readable file where every setting already has a default worth keeping. Work that already exists somewhere else — an issue backlog, a folder of documents, another spec-driven tool's output — should be something I can bring in rather than retype.

## Criteria

- **SETUP-1**  One command sets a project up with defaults that work and nothing I have to answer.
  - **SETUP-1.a**  Setting up does not switch on the change workflow; that stays a separate, deliberate decision.
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
