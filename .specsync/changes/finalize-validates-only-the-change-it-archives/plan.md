---
change: finalize-validates-only-the-change-it-archives
artifact: plan
---

# Plan

## Approach

Give finalize and archive a structural roster: active records as today, archived records parsed and checked from their own `state.json` bytes, with no per-archive Git history walk. Validate the history of the change being archived in full, as today, and of any archive the graph pass actually relies on. Move the per-archive history walk to `change audit --strict`, which names each failing archive.

## Out of scope

Read paths (`status`, `show`, `list`, `ship-status`), approve, reopen, adopt and metadata correction keep the full roster. No gate, bound, timeout, approval rule or persisted record changes. The default `change audit` stays active-only. The CI workflow and the Archive gate script do not change.

## Steps

1. Commit failing tests: a cold finalize whose Git processes do not grow with other archives, and a finalize that is not aborted by a hidden downgrade in an archive it does not rely on.
2. Add `list_all_changes_structural` and switch the two roster loads in `archive_change_with_options` to it.
3. Validate an archived record's own history in `AcceptedInputValidationMemo::authenticate` before relying on it.
4. Add `audit_project_with_strict` and pass the global `--strict` flag from `change audit`, with a test that it names a hidden downgrade in another archive.
5. Update specs, run the verify lane, `change check --commit`, and time finalize on this repository against the #842 build.

## Risks

A structural roster could let an archive with a broken history speak for the change being archived. The only ways another archive's evidence reaches a verdict about the target are dependencies (`load_change` already validates their history), semantic successors and sequence owners (both go through `AcceptedInputValidationMemo::authenticate`, which now validates that archive's history first). A broken history in an archive nothing relies on no longer stops finalize. `change audit --strict` reports it, so CI's strict audit step now carries the walk.

## Constraints consulted

change.3md "refusing gates install no memo and stay live"; REQ-change-119 and REQ-change-121 (cache and memo rules unchanged); REQ-change-audit-project-001 (default audit stays active-only); REQ-change-049 (ordinal collisions still refused at approve and audit); Leif: never weaken a gate for the change being finalized.
