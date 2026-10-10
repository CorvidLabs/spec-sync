---
hi: 1
families: [THREE]
---

# Three

## Intent

A module can keep what it learned in a text file next to its spec, with context on one plane and each lesson on a later plane. The spec itself stays markdown.

## Criteria

- **THREE-1**  I want a module's lessons and context in an optional 3md file beside the spec.
  - **THREE-1.a**  I want the spec to stay module.spec.md.
  - **THREE-1.b**  I want context.md to keep working until that module moves.
- **THREE-2**  I want one plane for context and later planes for lessons.
  - **THREE-2.a**  I want that file to stay text so I can read it in a pull request.
- **THREE-3**  I want that file named <module>.3md next to the spec.
