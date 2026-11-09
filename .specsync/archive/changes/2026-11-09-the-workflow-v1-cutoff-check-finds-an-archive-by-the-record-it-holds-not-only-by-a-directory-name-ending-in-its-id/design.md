---
change: the-workflow-v1-cutoff-check-finds-an-archive-by-the-record-it-holds-not-only-by-a-directory-name-ending-in-its-id
artifact: design
---

# Design

In `workflow_v1_record_present_in_cutoff_tree`, an archived record's candidates also include its own resolved package path, `find_change_dir(root, id)/state.json` made repository-relative, when it differs from the active path. That is the same list `workflow_version_state_paths` starts from. The record found there at the cutoff must still have the same id, workflow version 1 and origin `None` or `Some(1)`, so a different change's package can't satisfy it.

Nothing else changes: the cutoff listing, the non-canonical `-<id>` refusal, the persistent cache entry keyed by `(cutoff, id)` and only stored on presence, and the call site in `validate_workflow_v2_baseline`. Equivalence with the history walk: every path the walk could find that also existed at the cutoff is either the active path, a `*-<id>` entry of the cutoff's archive tree, or the record's own resolved path, so the candidate set now covers all three.
