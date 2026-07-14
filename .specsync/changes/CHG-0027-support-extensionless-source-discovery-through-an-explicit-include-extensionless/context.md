---
change: CHG-0027-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: context
---

# Context

SpecSync currently represents an omitted or explicitly empty `source_extensions` list as the same empty vector, which intentionally selects the default supported-language set. The shared TOML string-array parser also drops empty strings. Consequently, `source_extensions = [""]` cannot select extensionless files, and a project containing only executable files such as `bin/tool` can report vacuous zero-file coverage.

Issue #369 requires an explicit, backwards-compatible way to include extensionless source files. The setting must affect every source-discovery path, not only the coverage command, while leaving omitted and empty extension lists unchanged.
