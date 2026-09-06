---
change: let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived
artifact: requirements
---

# Requirements

- ADDED `REQ-change-095`: a module owns delivery paths beyond its spec's `files:` through `[modules."<name>"] owns`; succession for a predecessor entry signed under reserved exact owners is judged by the module that owns the path now, not by the frozen label.
- MODIFIED `REQ-change-020`: a changed input whose signed owners are all reserved exact labels requires one obligation from a successor whose module owns the path under the current configuration.
- MODIFIED `REQ-change-024`: the walk reads the claimants of a changed exact-only entry from the successors' declared obligations and covers it through any one that authenticates.
- MODIFIED `REQ-change-036`: the exact-only diagnostic names the supersede alternative wherever the configuration can grant the path, and a refused claimant of an exact-only entry is reported like a signed owner's successor with the frozen label as the owner.
- ADDED `REQ-config-013`: `owns` under `[modules."<name>"]` is parsed, typed, serialized, and never a source mapping.
- `types` Public API: `ModuleDefinition` documents `owns` beside `files` and `depends_on`.
