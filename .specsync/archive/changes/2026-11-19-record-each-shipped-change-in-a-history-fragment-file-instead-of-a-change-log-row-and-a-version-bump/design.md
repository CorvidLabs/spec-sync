---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: design
---

# Design

- **Fragment.** `history/<change-id>.md` beside the spec (the spec's own directory, resolved through the registry-aware `canonical_module_paths`), content `---\nchange: <id>\ndate: <UTC date>\n---\n\n<title>\n`. One prepared transaction writes it with the delta's spec and requirements text, so a failed check leaves neither.
- **Recorded.** A module records a change when its fragment exists or its Change Log names `<id>:`. `prepare_module_application` and the send-back reads use that one predicate. A recorded module gets nothing more; an unrecorded one gets a fragment, never a row or a bump.
- **Canonical file.** The fragment joins the spec and companions in `lifecycle_commit_scope`, `record_covers_project_path` and the acceptance-owner resolver. Send-back finds the pre-change tree from the commit that added the fragment, falling back to the row search for older changes, and names a dropped module's fragment for removal.
- **Validation.** `validate_spec_content_with_sources` (companion pass) checks each entry of `history/`: a regular `*.md` file, frontmatter `change:` equal to the stem and a valid change id, a `YYYY-MM-DD` `date:`, one non-empty summary line; and no change both in a fragment and in a Change Log row. The snapshot digest frames every `history/` entry's name and bytes.
- **Rendering.** `specsync history <module>` (in `src/commands/history.rs`, beside `doctor` under `cmd_change`) prints the Change Log table as written, then one row per fragment in the table's columns: a Version column counts on from the frozen `version:` (integer or the patch of `major.minor.patch`), Date is the fragment's date, the description column is `<id>: <summary>` with `|` escaped. `--format json` gives `module`, `version`, `rows` and `fragments`.
- **Rejected:** converting existing rows into fragments (the issue says no row is rewritten); a freeze marker in each spec (it would touch every spec once and records nothing a reader needs); computing `version:` into the spec (that write is the conflict).
