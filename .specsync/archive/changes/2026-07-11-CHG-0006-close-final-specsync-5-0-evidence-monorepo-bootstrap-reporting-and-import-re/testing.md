---
change: CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-change-014 | 74 focused lifecycle tests cover stale accepted inputs across unrelated commits, required legacy evidence, archive revalidation and attribution, subproject/custom-spec roots, canonical spec coverage, adoption bootstrap ancestry/removal, no-spec contradiction in accepted state, symlink ancestors/leaves, and mutation-free import rejection |
| REQ-cmd-comment-002 | Integration regression with SDD enabled, no canonical specs, and an SDD-only failure rendered in the PR comment |
| REQ-cmd-init-004 | Integration regression proving non-Git initialization produces a usable check while Git initialization retains enforcement |

Local evidence: 1,550 unit tests and 195 integration tests passed; Clippy passed with warnings denied; Astro reported zero diagnostics; 21 documentation tests passed; the 34-page documentation build, TypeScript compile, VSIX package, and optimized Rust build all passed. The strict SDD gate and executable lifecycle examples run again after canonical acceptance and evidence migration; GitHub checks validate the pushed result.
