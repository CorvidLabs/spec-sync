---
change: a-module-that-owns-a-changed-file-may-be-declared-without-a-delta
artifact: plan
---

# Plan

## Approach

Let a declared module go without a delta when it owns one of the change's declared files, and
make every per-module delta read skip modules with no delta file.

## Out of scope

Directory scopes as proof of ownership (only declared files that exist count); an explicit owner
flag; changes with no delta at all, which stay `--no-spec-change`.

## Steps

1. Red tests in send_back_for_rescope.rs.
2. `code_only_modules` and the relaxed check in `validate_delta_files`; `delta_modules` at every
   delta read.
3. deltas.md and quickstart.md.
4. REQ-change-161 delta once its wording is checked; full unit suite, fmt and clippy.

## Risks

A delta read that still iterates `affected_specs` would fail on a code-only module; the full
suite and the end-to-end test cover the lifecycle. A deleted approved delta must not pass as
code-only: the approval binding reports it as changed.

## Constraints consulted

The approved-delta binding (REQ-change-111, REQ-change-113), canonical ownership at approve and
finalize, and the send-back orphan rule for applied text.
