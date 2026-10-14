---
change: fix-custom-cargo-library-discovery-and-scoped-npm-generation-for-specsync-6-1-1
artifact: tasks
---

# Tasks

- [x] Add failing regressions for both late findings.
- [x] Fix Cargo library targets and scoped-name output.
- [x] Prepare 6.1.1 distribution pins and notes.
- [x] Fix the shared-library scan omission outside a flat member and retain exact ownership through nested workspaces. The original nested parser case was a retention check, not a failing-before discriminator.
- [x] Add a genuinely nested escaping-library CLI regression: it fails against the original committed PR816 subject; the parser test now covers both internal retention and the external shared root.
- [x] Repair cross-member propagated scan boundaries and add a CLI matrix with two sibling targets and same-named error/config files. That matrix fails on main and the first corrected candidate (v1), but passes on the original committed PR816 subject.
## Remaining release gates

Verification, provenance and human implementation review precede archival and merge. Then qualify one immutable candidate on Ubuntu and macOS before publication. These gates remain pending until their evidence is observed.

The exact-root corrective draft also needs real expanded validator-scope approval
and revised approved requirement/delta text. Existing approvals and state are
unchanged; local corrective preparation does not supply that human decision.

## Independent workspace/root-target review correction

Narrow workspace aggregation preserves ordinary per-package files and stable init.
Root-level Cargo targets use exact retained file selections instead of a project-wide
scan. Before/after CLI ownership and unrelated-output regressions, plus retained
file identity/resource-bound tests, qualify the correction. Fresh full verification
and independent review remain required; no human acceptance is inferred.

## Local v3 preparation after independent review

- [x] Preserve the complete reviewed v2 candidate, reports and binary.
- [x] Reproduce root-package cross-member library/binary scan splitting on the preserved v2 binary; this layout also affected the committed PR816 baseline.
- [x] Extend member-boundary filtering to root-package selections and add exact-owner regression controls.
- [x] Document literal manifest-resolved workspace scope and exact-file source_dirs minimum reader 6.1.1; released 6.1.0 skips those entries.
- [x] Complete final v3 native qualification and independent Grok review on the frozen candidate.
- [x] Obtain actual human acceptance of the expanded definition and tested implementation: Leif answered "Accept the tested fix; proceed through the release gates" on October 14.

## October 14 delivery update

The earlier pending-decision paragraphs above preserve the preparation history. Leif has now explicitly accepted the exact tested root-file, workspace ownership and validator scope. Current definition metadata and delta text are amended to that accepted contract; prior approval history remains intact. Supported `change approve`, product-tip verification, provenance, scoped review and same-PR archival still supply their own real evidence. No hosted matrix, merge or publication is implied by acceptance.
