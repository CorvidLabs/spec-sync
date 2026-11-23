## ADDED

### REQUIREMENT REQ-change-archived-requirement-loss-001

`change audit` on a SpecSync-managed branch SHALL still check for unexplained loss of canonical requirements when no active change workspace remains. On a Git branch, `check --strict` SHALL use the same comparison, including on a project that does not use SDD audit (CHANGE-17.d). The read-only comparison SHALL use REQ-change-169 for resolving a usable local default ref, selecting the canonical files, comparing tracked IDs across files, and recognizing approved removals. Its project-level reference SHALL be the merge base of HEAD and the usable default ref. A removal SHALL be credited only to an approved change on the branch, including a change archived there after the reference. Archives already present at the reference SHALL grant no new removal authority. The active-change history and base fallback in REQ-change-169 SHALL continue to apply to its lifecycle checks.

Both command families SHALL report each unexplained loss once, naming the ID, original canonical file and a `git show` command that displays the original block for restoration. A known default ref with unreadable comparison history SHALL cause a refusal that identifies the unavailable history and gives fetch guidance. With no usable default ref in a Git checkout, the branch-level comparison SHALL report that it was skipped. These branch findings SHALL be part of the corresponding text and JSON results.

Acceptance Criteria

- After finalization, a merge resolution drops a canonical requirement. Both audit and strict check exit unsuccessfully and identify the same ID, original file and restore command. The full-history required Lifecycle invocation rejects the archived tip.
- A separate control from the same base retains the requirement after finalization. Audit and strict check pass.
- Both command families accept an ID moved between tracked canonical files and a removal approved on this branch. A historical archive that already existed at the comparison base cannot justify a later deletion.
- A Git-backed project running only strict check reports the loss with the same restoration detail. Ordinary check keeps its drift behavior.
- A known default ref without enough history to find the merge base produces a refusal with fetch guidance. A checkout without a usable default ref reports a skipped comparison.
