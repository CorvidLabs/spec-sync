---
spec: generator.spec.md
---

## Tasks

- [ ] Add `--force` flag to overwrite existing specs (with confirmation prompt)
- [ ] Support interactive mode that asks the user to confirm/edit each generated spec before writing
- [ ] Populate companion files with real content from AI instead of empty templates
- [ ] Add `--dry-run` flag to preview what would be generated without writing files

## Done

- [x] Template-based spec generation with language-specific templates
- [x] AI-powered spec generation with fallback to template
- [x] Custom template support (`_template.spec.md`)
- [x] Companion file generation (tasks.md, context.md)
- [x] Module detection from config, subdirectories, and flat files
- [x] Flat file entry point exclusion (main, lib, mod, index, app, `__init__`)

## Gaps

- Generated specs from templates have placeholder content that scores low on the quality rubric
- No way to regenerate a single spec without deleting the existing one first

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
