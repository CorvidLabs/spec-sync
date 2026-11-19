## ADDED

### REQUIREMENT REQ-cmd-change-038

`change status` SHALL show each delta rebase recorded under the effective definition approval (REQ-change-161), naming the module, the requirement and when it was rebased, in text and in JSON.

Acceptance Criteria
- After `change check` rebases a delta, text `change status` prints a line naming the module and requirement as rebased over main's edit, and `--json` lists the rebase under `summary.delta_rebases`.
