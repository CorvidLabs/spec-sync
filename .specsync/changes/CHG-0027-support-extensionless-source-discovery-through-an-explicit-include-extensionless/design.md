---
change: CHG-0027-support-extensionless-source-discovery-through-an-explicit-include-extensionless
artifact: design
---

# Design

Add `include_extensionless: bool` to `SpecSyncConfig`. A dedicated boolean avoids overloading the empty string and preserves the established meaning of both omitted and explicitly empty `source_extensions`: use the default supported-language set.

Introduce one shared source-extension predicate that first recognizes a path with no extension when the boolean is enabled, then delegates ordinary extension matching to the existing predicate. All configuration-driven scanners use this shared predicate. Dotfiles follow `std::path::Path::extension` semantics and therefore count as extensionless; directory and exclusion filtering remains unchanged.

The canonical TOML writer omits the false default and writes `include_extensionless = true` when enabled. Serde's existing camel-case compatibility exposes `includeExtensionless` in legacy JSON. No configuration schema version bump is required because the field is additive and defaults safely.
