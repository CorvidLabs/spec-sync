---
change: the-workflow-v1-cutoff-check-reports-an-unreadable-archive-tree-as-a-read-failure-not-as-an-absent-change
artifact: design
---

# Design

In `workflow_v1_record_present_in_cutoff_tree`, when `ls-tree -z --name-only <cutoff>:<archive root>` fails, tell "no archive root at the cutoff" apart from "an archive tree that can't be read". Ask whether the cutoff commit's tree lists the archive root: `ls-tree --full-tree -z --name-only <cutoff> -- <archive root>`. The command's batch reader already answers that from tree objects (#869), and it falls back to Git when a tree is missing.

- It succeeds and doesn't list the archive root: there is no archive at the cutoff. No archived candidates, the same verdict as today.
- It lists the archive root, or it fails: return `Err("the archive tree at the trusted pre-v2 cutoff <cutoff> could not be read: ...")` with Git's message. The error propagates through `validate_workflow_v2_baseline` as any read failure does, and nothing is cached, because only a conclusive presence is ever stored.

A successful listing takes the same path as today.
