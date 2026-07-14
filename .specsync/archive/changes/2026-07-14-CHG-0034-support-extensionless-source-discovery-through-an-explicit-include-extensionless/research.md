---
change: CHG-0034-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: research
---

# Research

`parse_toml_string_array` intentionally filters empty values and is shared by unrelated configuration arrays. Preserving empty strings globally would change their semantics and still leave an empty configured extension list ambiguous with the default. The scanner already treats an empty extension vector as all supported languages, so a separate boolean is the smallest unambiguous compatible contract.

Source-extension filtering occurs in validator coverage, generator discovery, new/scaffold/wizard flows, diff reporting, and terminal output. Updating only coverage would create inconsistent module ownership and generation behavior, so all call sites must share the new predicate. WalkDir yields both files and directories, so wizard discovery also needs an explicit regular-file guard before filename matching.
