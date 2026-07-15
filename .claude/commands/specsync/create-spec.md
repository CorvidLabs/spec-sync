---
description: Scaffold a new spec-sync module spec from a module name or a natural-language feature description (full scaffold by default, or minimal with --minimal)
argument-hint: <module-name-or-description> [--minimal]
---

Create a new spec-sync module spec.

Arguments: `$ARGUMENTS`

1. Read the complete arguments above. Remove each standalone `--minimal` flag
   (in any position) and remember that minimal mode was requested. Preserve
   the complete remaining input for classification; do not extract a module
   name yet. If nothing remains, ask the user for a module or description.
2. Classify the complete remaining input as one of:
   - **A bare module name** — the entire input is one identifier with no
     whitespace, such as `auth-service` or `billing`. Use it as-is.
   - **A free-text feature description** — any quoted or unquoted sentence or
     phrase describing what to build, e.g. `"I want a feature that lets users
     export their data as CSV"`. Only after making this classification, invent
     a short, kebab-case module name that captures the idea (e.g. `csv-export`).
     Never use only the first word as the module name. If the right name is
     ambiguous, ask the user to confirm or rename it before continuing. Keep
     the complete description at hand — you'll use it in step 5.
   Flag position does not change classification:
   - `--minimal billing` and `billing --minimal` both select the bare module
     `billing` in minimal mode.
   - `--minimal I need CSV export` and `I need CSV export --minimal` both keep
     the complete description and derive a name such as `csv-export`, not `I`.
3. If minimal mode was requested, run:
   ```
   specsync new <module-name>
   ```
   This creates a minimal spec only (no companion files).
4. Otherwise (default), run:
   ```
   specsync scaffold <module-name>
   ```
   This creates the spec, companion files (`tasks.md`, `requirements.md`,
   `context.md`, `testing.md`, and `design.md` if `companions.design` is
   enabled), a registry entry, and auto-detects related source files.
5. Open the newly created `specs/<module-name>/<module-name>.spec.md` and fill
   in the `Purpose`, `Requirements`, and `Public API` sections. If a free-text
   description was given in step 2, use it directly to draft these sections —
   ask clarifying questions if it's underspecified, but do not leave the
   sections as unfilled placeholder text. Do the same for `requirements.md`
   (acceptance criteria) and `tasks.md` (initial task breakdown), if present.
6. Run `specsync check` to confirm the new spec passes validation.
