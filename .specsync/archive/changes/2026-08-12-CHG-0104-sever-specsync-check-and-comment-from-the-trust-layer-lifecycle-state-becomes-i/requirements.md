---
change: CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i
artifact: requirements
---

# Requirements

## REQ-cmd-check-004 — lifecycle state is informational in `check`

`specsync check` SHALL NOT determine its exit code from SDD lifecycle state.

- It SHALL report the number of active changes as an informational line.
- It SHALL emit shape warnings to stderr for workspace files that cannot be parsed
  or that record an illegal state.
- Its exit code SHALL be determined solely by spec validation results, the
  effective enforcement mode, `--strict`, and `--require-coverage`.
- Lifecycle gating remains the responsibility of the `change` verbs and
  `specsync change audit`.

## REQ-cmd-comment-004 — comment reports specs, not trust

`specsync comment` SHALL NOT fold SDD lifecycle errors or warnings into the
spec-check results it reports, and lifecycle state SHALL NOT contribute to its
exit code.

## REQ-change-058 — no quiet-output lifecycle check

The lifecycle check SHALL expose exactly one configured-command output behavior. The
quiet-output variant, which existed only so PR comments could consume lifecycle findings
without child output contaminating stdout, SHALL be removed along with its selector type
rather than left unused.
