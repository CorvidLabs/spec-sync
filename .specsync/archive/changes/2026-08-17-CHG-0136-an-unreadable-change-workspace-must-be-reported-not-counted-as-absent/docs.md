---
change: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
artifact: docs
---

# Docs

A CHANGELOG entry lands under `## [Unreleased]` → `### Fixed`, covering both the user-visible
behaviour change and the two consequences a reader would otherwise have to discover.

## What the entry must state, and does

- **The observable change.** `list` and `status` no longer print the empty-project line over a
  corrupt sibling; they name the unreadable workspace with its path and exit non-zero.
- **The mechanism**, because "one bad workspace hid everything" is not obviously two defects:
  enumeration aborted on the first failure, *and* the resulting error became an empty list.
- **That this is also #603.** A reader tracking mixed-version corruption needs to know no schema
  version moved and none needed to. The downgrade was already detected and was being discarded.
- **The three further callers.** The pull-request diff base, sibling-in-flight reporting, and
  `ship`'s target inference were all reading the same empty roster as fact. `ship` writes commits.

## Breaking-change disclosure

Two behaviours change for existing consumers, and both are called out rather than left to be
found:

1. **Exit status.** `list`, `status` and `ship-status` now exit non-zero on a tree with an
   unreadable workspace. No correct script depended on the previous behaviour, because the
   previous behaviour was to report such a tree as empty and exit 0 — but the change is real and
   a CI job that treated `list` as infallible will now fail on a corrupt workspace, which is the
   point.

2. **JSON shape, conditionally.** `list` and `status` keep the bare array while every workspace is
   readable — unchanged for every project not already being lied to. A degraded roster becomes an
   object with `changes`, `unreadable` and `error`. The entry states this explicitly so a
   consumer knows the shape is conditional rather than discovering it during an incident.

No user-facing guide or README section describes the roster's exit status, so nothing outside the
CHANGELOG requires updating. `--help` text is unchanged: the commands' descriptions remain
accurate.
