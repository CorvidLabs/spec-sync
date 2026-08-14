---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: context
---

# Context

A source file with unresolved conflict markers passes `check` with exit 0.
Measured: `sub` on the HEAD side, `mul` on the other, spec documenting both:

    ✓ 3/3 exports documented ; 1 specs checked: 1 passed ; exit 0

The extractors parse both sides of the hunk as ordinary declarations, so the
union satisfies the spec. spec-sync green-lights a tree that does not compile.
Spec bodies carrying markers pass the same way.

The obstacle is why this was not fixed sooner. `git grep -n '^<<<<<<< '` returns
twelve lines across three files in this repository, and every one is a complete,
well-formed triple inside a raw string literal in test code:

    src/merge.rs                    9 lines  — files:-mapped by specs/merge
    src/exports/ast/rust_lang.rs    1 line   — files:-mapped by specs/exports
    tests/integration/commands.rs   2 lines  — not mapped

The first two are scanned on every real `check` run against this repo. So a
guard that fires on marker-shaped content makes spec-sync fail its own tree, and
a structural check does not help: these are syntactically perfect triples. The
only thing distinguishing them is that they sit inside a string literal, which
needs a lexer, and thirty-plus extractors are regex-only with `ParseMode::Regex`
the default.

Ruled out: refusing on marker text. Ruled out: refusing on a well-formed triple.
Both red-light this repository.
