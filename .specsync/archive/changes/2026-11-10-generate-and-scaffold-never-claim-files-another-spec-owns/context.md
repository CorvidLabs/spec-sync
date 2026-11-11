---
change: generate-and-scaffold-never-claim-files-another-spec-owns
artifact: context
---

# Context






GitHub issue #611. Verified on main 49d2326a: `specsync scaffold foo` wrote a spec claiming
src/foo.rs already owned by specs/special. `find_creation_module_sources` and
`find_single_source_fallback` scan the filesystem without consulting existing specs' `files:`
lists; the coverage-aware path (`module_files_from_coverage`) self-corrects, the direct scans
do not.
