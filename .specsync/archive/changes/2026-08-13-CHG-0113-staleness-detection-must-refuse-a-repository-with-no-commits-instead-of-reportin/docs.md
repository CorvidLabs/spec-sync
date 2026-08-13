---
change: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry, noting that the no-repository case was already handled and that an
unborn `HEAD` was simply not recognised as the same precondition.

## Behaviour change

| tree | before | after |
|---|---|---|
| git repo, no commits | `✓ All specs are up to date`, exit 0 | refused, exit 1, naming the missing commits |
| git repo with commits | — | unchanged |
| not a git repository | refused, exit 1 | unchanged |

A project that runs `stale` immediately after `git init` will now see a refusal instead of
a green line. That is the intended signal: there is no history to derive a verdict from.

## New public API

| Symbol | Spec |
|---|---|
| `git_utils::has_commits` | `specs/git_utils` |
