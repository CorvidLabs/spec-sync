---
change: CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest
artifact: context
---

# Context

Gradle Kotlin discovery derived a module name by taking the FIRST path segment,
so `com.example.foo` and `com.example.bar` both became a module named `com`, and
`generate` wrote `specs/com/com.spec.md` for an entire package tree.

A first attempt replaced first-segment with LAST-segment. It was rejected: on a
monorepo laid out as `packages/<name>/src/main/kotlin/...` the leaf rule
collapses everything into one module named `src`, and `generate` then writes a
single wrong spec for the whole tree. Same defect, different name. Its own risk
note called leaf collisions "rare" and "the least-bad outcome" — that is the
standard monorepo layout.

The rejection forced the real question, and the answer reframes the bug: what
rule does discovery use for every OTHER language?

    Cargo      [package] / [[bin]] name
    Swift      .target(name:)
    npm        name (workspace directory in a monorepo)
    pubspec    name
    pyproject  [project] / [tool.poetry] name
    Go         last segment of the module path

All manifest identity. Directory scanning is only the no-manifest fallback.

Gradle ALREADY uses the manifest rule for `settings.gradle` includes. The defect
is that a single-project build never inserts a module, so the shared fallback
sees `src/main/kotlin/com` and names a module from a path segment. So this was
never a question of which segment to pick — it is that Gradle was the one
language deriving identity from a path at all.
