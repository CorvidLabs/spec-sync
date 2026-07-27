---
change: CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base
artifact: docs
---

# Docs

## Canonical Documentation

Update `specs/merge/merge.spec.md` and its companions to describe diff3 parsing, maximum numeric
versions, lossless list unions, conservative row/scalar handling, and all-or-nothing writes.

## Pull Request

Rewrite PR #448's title and body so they do not claim partial resolutions are persisted. Include
the issue #427 closure, targeted test matrix, lifecycle evidence, and full verification results.

## Release Notes

No new CLI surface is introduced. Note the compatibility change that ambiguous merge conflicts
now remain manual instead of selecting an arbitrary side.
