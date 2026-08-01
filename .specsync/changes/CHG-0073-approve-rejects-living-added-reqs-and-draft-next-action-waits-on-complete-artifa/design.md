---
change: CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa
artifact: design
---

# Design

## next_action ordering (draft)

When the interview is complete, evaluate artifact completeness before recommending approve. Reuse validate_artifacts (incomplete HTML TODO comment stubs / empty body) and list incomplete paths in the guidance string.

## Living ADDED detection

During validate_delta_files (invoked by approve), for each ADDED requirement item, check living requirements.md for a matching requirement heading. Fail closed with a MODIFIED remediation hint. Materialize still retains the same cannot-add-existing-block defense.

## Command adapter

text_mode_next_action takes the project root and applies the same lightweight completeness check so human text show/list does not recommend approve for incomplete drafts.
