---
change: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
artifact: context
---

# Context

#474 reads as a display bug — "`score` ignores `export_level` type and grades
against public members". It is not. Measured on a project configuring
`export_level = "type"`:

    check --strict   rc=0   ✓ 2/2 exports documented
    score --json     api 8/20, total 88
                     "API coverage (-12pts): 3 undocumented export(s) — id, name, find"

Two commands, one tree, incompatible answers about what the module's API IS.
`check` used the configured level; `score` did not, and then graded the spec
against a surface the project never claimed.

The cause was a convenience wrapper — `exports::scan_exported_symbols(path)` —
that hard-codes the export level and parse mode. Grepping for its callers turned
a scoring bug into five:

- `score` — the reported symptom.
- `new` — **generated specs its own validator then rejected.** Activate a
  generated spec and run `check`: `Spec documents 'id' but no matching export
  found in source`. The tool created work for itself to refuse.
- `generate` — same, on the retained path.
- `scaffold` — same.
- `diff --json` — reported `"new_exports": ["id","name","find"]` as drift for
  symbols the contract never claimed. That lands in PR comments.

The wrapper also hard-coded `parse_mode`, so a project configuring `ast` got AST
parsing in `check` and regex everywhere else. One config, several answers.
