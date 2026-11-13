---
change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
artifact: research
---

# Research

- `load_change` reads `state.json` and names its path on a parse error, but checks no schema
  version. `load_approvals` and `load_verification` drop the path. `load_verification_attempt_ledger`
  refuses `schema_version != 1`, but only on the paths that load it (check, verify).
- The newer-workflow message already exists: "<file> was written by a newer SpecSync (workflow
  version N); upgrade specsync to read it". The schema message reuses its shape.
