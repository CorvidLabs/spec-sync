---
change: split-configuration-loading-parsing-and-discovery-into-focused-private-modules
artifact: research
---

# Research

Read config contracts and Human Intent constraints before implementation. The config file currently mixes directory discovery, retained parsing, repair validation, strict and permissive loading, legacy diagnostics, two compatibility parsers and migration serialization. These provide existing semantic extraction boundaries. Exact-byte checked parsing validates before calling compatibility parsing; supplied source lists use lazy fallback. JSON's invalid repository sentinel is intentional. Existing tests cover migration, source detection, malformed config and TOML literal edge cases.

Pristine focused run: 98 passed, 0 failed. Compiled inventory captured before edits. Independent review will verify declarations, bodies, constants, traits and cfg as well as the security-sensitive seams.
