# Lesson bundle: generate-and-scaffold-never-claim-files-another-spec-owns

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Generate and scaffold never claim files another spec owns
- **Kind**: BugFix
- **Specs**: generator, cmd_generate, cmd_scaffold
- **Paths**: src/generator.rs, src/commands/generate.rs, src/commands/scaffold.rs, specs/generator/, specs/cmd_generate/, tests/
- **Acceptance**: scaffold/new/generate filter candidate files against files already mapped in existing specs' frontmatter; a module whose files are all owned produces no spec; the MCP generate path behaves identically

## Evidence

- Verification commit: `787a34a892f4445f9dae7c50e635bcb16731dbb7`
- Base commit: `49d2326a8f80232b8fee94a5073c4c1a66fd2057`
- Verified by: `specsync check --spec cmd_generate --spec cmd_scaffold --spec generator`

## From the change's context.md

# Context






GitHub issue #611. Verified on main 49d2326a: `specsync scaffold foo` wrote a spec claiming
src/foo.rs already owned by specs/special. `find_creation_module_sources` and
`find_single_source_fallback` scan the filesystem without consulting existing specs' `files:`
lists; the coverage-aware path (`module_files_from_coverage`) self-corrects, the direct scans
do not.

## From the change's testing.md

# Testing


Integration: spec `special` owns src/foo.rs; `scaffold foo` writes no spec (or a spec without
src/foo.rs); `new foo` likewise. Unit: candidate filtering against owned files.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-generator-009 | integration `scaffold_never_claims_a_file_another_spec_owns`, `scaffold_single_source_fallback_skips_an_owned_file`; existing `scaffold_auto_detects_single_source_file` still passes |
| REQ-cmd-generate-004 | generate modes bind from coverage (unchanged path); integration generate suite 44/44 green |
| REQ-cmd-scaffold-006 | integration `scaffold_never_claims_a_file_another_spec_owns` |

## Where these lessons go

- `specs/generator/generator.3md`
- `specs/cmd_generate/cmd_generate.3md`
- `specs/cmd_scaffold/cmd_scaffold.3md`
