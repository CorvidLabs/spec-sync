---
change: CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa
artifact: docs
---

# Docs

Empty 5.0.1-era `.specsync/registry.toml` stubs (for example `version = 1` with an empty
`[modules]` table) no longer block module resolution. SpecSync treats those inert files as
absent and falls back to conventional `specs/<module>/<module>.spec.md` paths. Real registries
that name a project or map `[specs]` modules still parse normally; non-inert unparsable files
continue to fail closed.
