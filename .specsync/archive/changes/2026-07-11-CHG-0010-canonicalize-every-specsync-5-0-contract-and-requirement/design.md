---
change: CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement
artifact: design
---

# Design

Use one module-scoped delta for every affected spec. Legacy requirements receive additive stable IDs; no existing user story, acceptance criterion, constraint, or out-of-scope detail is removed. Modules with API or product-truth drift receive full `SPEC SECTION` replacements so acceptance can apply the future contract atomically.

For the 14 dependency mismatches, the delta replaces the human-readable Dependencies section with the preserved canonical content plus an explicit list of frontmatter additions. The implementation phase must update YAML `depends_on` separately because semantic section application does not own frontmatter.

Promote `cmd_migrate` from draft to stable as an explicit implementation edit only after its delta reconciles the eleven-step pipeline and verification evidence. Keep genuine future tasks open. Repair the missing `cmd_rules/context.md` header without broad companion rewrites.

Task cleanup is evidence-based, not mechanical. Six fully evidenced CLI/regression rows may close; four mixed rows must be split into a completed narrow claim and an open remainder. Remaining unchecked work is categorized under `Post-5.0 Roadmap`, `Test Debt`, or `Manual`. Repeated pending signoff templates become an informational note that acceptance remains governed by the change lifecycle; no approval is synthesized.

One narrow source behavior change is authorized: `config_to_toml` and the committed canonical config must use the exact version-neutral header `# spec-sync configuration`, protected by a focused inline regression test. Any other product behavior change requires an amended and reapproved definition.

Dependency inference is also corrected at its analysis boundary. Strip Rust comments and string literals before import
matching, then map top-level module paths to the spec owning `src/<module>.rs` or `src/<module>/mod.rs`. Preserve
the valid `cli -> cmd_change` edge while resolving `cmd_change -> cli_args`. Remove the real
`commands <-> rehash` cycle by making rehash use configuration and validator discovery directly.
