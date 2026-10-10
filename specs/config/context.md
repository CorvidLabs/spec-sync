---
spec: config.spec.md
---

## Key Decisions

- Current and legacy JSON/TOML layouts remain readable for migration.
- Retired AI key names are recognized only to emit value-safe migration guidance, then ignored.
- Configuration never interprets provider credentials or commands.
- `[modules."<name>"] owns` lists paths a module owns for change acceptance beyond its spec's `files:`
  — a file, or a directory that owns everything beneath it. It is parsed, validated as a string array,
  and serialized beside `files` and `depends_on`; a module carrying only `owns` still round-trips. The
  key is read by the change lifecycle alone: it is not a source mapping and takes no part in coverage.
- Source discovery recognizes supported language files plus default measurable HTML, HTM, and CSS content at the root or within top-level directories while preserving ignored-directory and empty-project behavior.
- Checked source-directory and manifest discovery surface malformed or unreadable Gradle settings;
  existing infallible entry points remain compatibility wrappers, with scan fallback for source dirs.
- A present legacy JSON `github.repo` accepts only a string (or explicit `null` as absent).
  Numbers, booleans, objects, and lists retain an invalid sentinel so repository resolution fails
  closed instead of falling back to Git auto-detection.
- Retained filesystem callers use the crate-private checked snapshot parser: JSON/TOML is parsed
  from exact supplied bytes, real TOML syntax and known field types are validated first, and no
  pathname is reopened.
- Capability callers can supply source-directory discovery from their retained project handle, so
  an omitted source list does not trigger ambient root traversal or observe a path replacement.
- Retained CLI discovery shares the canonical config precedence and lexical source classifier,
  reads selected bytes through its project capability, and does not scan an explicitly configured
  source root before normal validation reports path-safety findings.

## Files to Read First

- `src/config.rs` — stable API facade
- `src/config/discovery.rs` — manifest-aware and scan-based source discovery
- `src/config/checked.rs` — repair validation and exact-byte checked parsing
- `src/config/loading.rs` — file precedence, strict/permissive loaders, and local diagnostics
- `src/config/json.rs` — legacy JSON compatibility and invalid-repository sentinel
- `src/config/toml_config.rs` — compatibility section dispatch
- `src/config/toml_syntax.rs` — shared string, comment, and array syntax
- `src/config/serialization.rs` — canonical output and loss checks
- `src/types.rs`

## Current Status

Stable 5.0 secret-free configuration schema with checked discovery available to validation gates.
CHG-0063 adds fail-closed legacy GitHub repository shape validation, exact-byte checked parsing,
and capability-supplied source discovery for issue inspection. The exact checked JSON path rejects
a non-object `github` value and a non-string/non-null `github.repo` before the compatibility parser
can substitute an invalid sentinel. CHG-0063 exact-head remediation also shares config precedence
and source classification with retained CLI discovery while preserving the established fail-loud
fallback for malformed legacy CLI config.

## Private Implementation Organization

Configuration implementation and existing regressions now live in cohesive private owners behind
the existing public and crate-visible facade. Parsing consumes unchanged bytes in the same order;
supplied discovery remains authoritative and lazy fallback remains unchanged. Shared TOML syntax
helpers serve both the compatibility parser and serializer. Owner-local tests keep internal
helpers private; the source split changes no accepted formats, warning values, refusal behavior,
filesystem authority, defaults, or migration output.
