---
change: split-parser-frontmatter-issue-validation-symbols-and-sections-into-focused-private-modules
artifact: context
---

# Context

The owner explicitly requested breaking up spec-sync files above 1000 lines where practical, continuing the work, and creating a branch and commit. This package applies that existing-behavior refactor to the 2409-line parser module. Authorization provenance: owner messages “focus on breaking up the files in spec-sync so we don't have 10k+ or 5k+ or 1k+ if possible..”, “and continue on spec-sync”, and “amke a fucking branch and commit”. The agent records scope only; no human implementation review is claimed.

The parser contract already distinguishes permissive metadata compatibility from security-sensitive real-YAML issue discovery. Consumers keep their existing imports. Existing documented gaps remain outside this extraction.
