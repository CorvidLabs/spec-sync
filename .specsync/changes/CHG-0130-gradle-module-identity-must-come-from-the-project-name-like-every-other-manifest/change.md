---
id: CHG-0130-gradle-module-identity-must-come-from-the-project-name-like-every-other-manifest
state: implementing
type: bug_fix
base_commit: 0edba080ab9fee5879e9cae903ea2abb3b97a250
---

# Gradle module identity must come from the project name like every other manifest, not from a source path segment, because both the first and last segment collapse a whole tree into one module

## Intent

Gradle module identity must come from the project name like every other manifest, not from a source path segment, because both the first and last segment collapse a whole tree into one module

## Affected Canonical Specs

- `manifest`
- `validator`

## Acceptance Criteria

- A single-project Gradle build is named from a literal rootProject.name, or from the project directory when that is unset, matching Gradle's own default. A multi-project build continues to use its include names. No child of a JVM source root such as src/main/kotlin is treated as a module, so a package hierarchy no longer collapses into a module named after any one of its segments. A monorepo laid out as packages/<name>/src/main/kotlin does not collapse into a single module. Discovery for every other language is unchanged.

## No-spec Rationale

Not applicable
