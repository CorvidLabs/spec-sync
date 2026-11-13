---
change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
artifact: design
---

# Design

- `read_lifecycle_json::<T>(path)`: read, then parse. Every error is `<path>: <cause>`, and a path
  already in the cause is not repeated. The lifecycle loaders use it.
- `ensure_lifecycle_files_supported(root, record)`: for each lifecycle file present in the change
  directory, read only its top-level `schema_version`. Above what this build supports, it refuses
  with `<path> was written by a newer SpecSync (schema version N); upgrade specsync to read it`.
  `load_change` (so status, show and every verb) and the audit's per-change pass call it.
