---
change: split-configuration-loading-parsing-and-discovery-into-focused-private-modules
artifact: design
---

# Design

The facade retains external API paths. Discovery owns root scanning and manifest fallbacks. Checked parsing owns exact snapshot dispatch, JSON shape checks and TOML type checks. Loading owns config precedence, BOM-tolerant reads, permissive/strict loading and retired local diagnostics. JSON and TOML owners preserve compatibility parsing. Shared TOML syntax owns string escaping, comments and array token handling. Serialization owns canonical output and loss refusal. Private sibling seams are only as visible as their production consumers require.

Owner-local tests avoid widening helper visibility solely for tests. No complete function will be split to meet a line count.
