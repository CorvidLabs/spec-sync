## ADDED

### SPEC SECTION Internal Organization

Configuration retains its public and crate-visible API paths through a small facade. Private owners separate discovery, exact-byte checked validation, file loading and legacy diagnostics, JSON compatibility, TOML compatibility, shared TOML syntax, and canonical serialization. Existing tests are grouped beside their owners. The split preserves precedence, strict-loader refusal, supplied-source authority, checked versus compatibility GitHub handling, value-safe retired-key warnings, exact literal parsing, and lossless migration without introducing new public behavior.
