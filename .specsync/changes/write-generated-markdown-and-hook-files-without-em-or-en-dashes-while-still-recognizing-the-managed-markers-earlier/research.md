---
change: write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier
artifact: research
---

# Research

## Which generated strings are also read back

| String | Written by | Read by | Present in existing installs? |
|--------|-----------|---------|-------------------------------|
| Hook begin sentinel | `wrap_snippet` (`src/hooks.rs:516`) | `remove_section_from_file` (`src/hooks.rs:827`) | yes, every block since #300 |
| Hook end sentinel `<!-- specsync:hook:end -->` | same | `src/hooks.rs:828` | yes, no dash, unchanged |
| `Spec-Sync Integration` / `Spec-Sync Rules` | snippet headings | `is_installed`, install no-op check, uninstall marker | yes, no dash, unchanged |
| Instruction snippet bodies | `install_markdown_snippet` | exact-snippet fallback (`position_of_slice`, `src/hooks.rs:836`) | yes |
| `PRE_COMMIT_HOOK` body | `pre_commit_block`, inside project-keyed markers | unmarked legacy comparisons (`:671`, `:679`) | 6.1.1 body only inside markers; unmarked hooks carry the v5.x text, which has not matched since #774 |
| Pre-commit project-keyed markers | `pre_commit_markers` | install, uninstall, status | yes, no dash, unchanged |
| Compaction summary range and placeholders | `compact_spec_changelog` (`src/compact.rs:313`) | `summary_metadata` (`:500`, `:503`) | yes, in every compacted changelog |
| Lesson-bundle heading | `write_lesson_bundle` | nothing | yes, in frozen archives |
| `.specsync/.gitignore` header | `init`, `migrate` | nothing (existence only) | yes |
| create-spec command text | `agents install` | digest manifest only | yes, digest-tracked |

The first, fourth and seventh rows are the compatibility surface. The fifth row looks like one
but is not: no release wrote an unmarked hook with the current text, so changing it strands
nothing. Every other row is write-only.

## Alternatives considered

1. **Strip the characters at write time in one helper** (for example inside
   `atomic_write_durable`). Rejected:
   - It would also rewrite user text that the lesson bundle copies verbatim, which the issue
     scopes out.
   - It would change the bytes of files SpecSync hashes, such as specs, so evidence digests would
     move under users.
   - It hides the literal from the code reader. The test below gives the same guarantee without
     touching user content.
2. **Match the hook begin sentinel by the prefix `<!-- specsync:hook:begin`.** Simpler, and it
   would also accept future wordings. Rejected for now:
   - It widens recognition to hand-written begin lines a user may have edited.
   - Uninstall deletes everything between begin and end, so recognition should stay exact.
   An explicit list of the two known forms is the conservative reader. The open question in
   `tasks.md` asks the human to choose.
3. **Keep a frozen copy of each 6.1.1 instruction snippet for the exact-snippet fallback.**
   Rejected as unnecessary:
   - That fallback exists for pre-sentinel installs, which predate #300 (2026-07-02).
   - The snippet text has changed since then (the 3md lesson wording, #808, 2026-10-10), so the
     fallback already matches only the current text.
   - A 6.1.1 block always carries both sentinels and is removed by the first strategy once the
     old begin line is recognized.
   The heuristic fallback stays as the last resort.
4. **Rewrite existing 6.1.1 hook blocks on `hooks install`.** Rejected. Installation is
   documented as a no-op when a block is present, and auto-updating on upgrade is listed out of
   scope in `specs/hooks/requirements.md`. Users can run `hooks uninstall` then `hooks install`.
5. **Compaction range separator.** Candidates:
   - ` to `: reads as prose and is unambiguous for ISO dates.
   - `..`: compact, but reads as code.
   - ` - `: splits ISO dates correctly, because their hyphens carry no spaces. It is still the
     usual ASCII stand-in for the dash being removed. Whether the CorvidLabs docs rule accepts it
     in prose is not stated in the issue, so it is not chosen.
   - Chosen: ` to `. The reader splits on the first ` to `, and falls back to the legacy
     separator.
   - Residual: a first cell whose own text contains ` to ` would truncate the recorded range
     start. First cells are dates in every generated and documented changelog, so this is
     stated, not guarded.
6. **Compaction placeholder.** Candidates: an empty cell, `-`, or `n/a`. An empty cell
   renders, but it is easy to produce by accident when hand-editing. `-` is chosen. The
   provenance marker, not the placeholder, is what makes a row tool-owned.

## Punctuation chosen for prose

| Shape in 6.1.1 | Replacement | Example |
|----------------|-------------|---------|
| Definition list item ``- `x` `` dash text | colon | ``- `specsync check`: validate all specs against source code`` |
| Bold definition ``- **`tasks.md`** `` dash text | colon | ``- **`tasks.md`**: Work items for this module.`` |
| Parenthetical aside in a sentence | comma or semicolon | `These are permanent invariants, not tasks; do not check them off.` |
| Heading with an id | colon | `# Lesson bundle: <id>` |
| Comment header | colon | ``# spec-sync 5.0: generated by `specsync init` `` |
| Hook begin sentinel | semicolon | ``(managed by `specsync hooks`; do not edit inside)`` |

The begin sentinel uses a semicolon, as the issue proposes. A colon after a backtick span inside
an HTML comment reads as a label.
