---
change: make-the-lost-requirements-check-read-only-the-reference-s-canonical-files-spare-archived-changes-report-beside-a
artifact: design
---

# Design

- **#1112 (REQ-change-175).** `lost_requirements_check` builds the reference's scope on its own (`add_reference_requirement_scope` into an empty `RequirementScope`). Only its pathspecs name the files diffed and read at the reference. Presence in the working tree keeps the union of today's scope and the reference's, so a requirement moved into a newly canonical file is still found.
- **#1113 (REQ-cmd-change-045).** `lost_requirements_refusal`, which only `change ship-status` calls, returns nothing for an archived or retired change. A draft, approved, implementing, verifying or accepted change keeps the blocker.
- **#1116 (REQ-change-176).** In `change audit`, the loss check for an approved, implementing or verifying change runs on its own, no longer behind the approval, delta-file, delta-body and delta-base checks. An approval or delta that cannot be read already accounts for no removal (`add_approved_removals`).
- **#1117 (REQ-change-177).** `working_tree_requirement_blocks` also returns each scope directory it cannot list and each requirements file it cannot read, for any reason other than not existing. When the check would report a loss and that list is not empty, it refuses with the existing "cannot tell whether this branch lost canonical requirements" form, naming each path with its error and the way out: make it readable, then run `specsync change check <id>` again. With nothing lost it says nothing.
