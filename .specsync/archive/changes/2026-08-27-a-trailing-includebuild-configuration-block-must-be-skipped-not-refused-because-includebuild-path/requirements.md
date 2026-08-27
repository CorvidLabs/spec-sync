---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: requirements
---

# Requirements

## Must

- `includeBuild(path) { dependencySubstitution { … } }` parses when `path` is one complete string
  literal beneath the project root — multi-line (the reported form), one line, Groovy single quotes,
  `settings.`-qualified, with a trailing `;`, and inside a conditional block.
- An accepted composite build contributes **no module and no source directory**. A sibling
  `include(":app")` is still discovered, and nothing beneath the included build's path is measured
  as part of the root build.
- A real Gradle project written this way runs `coverage` successfully, reports its modules, and
  emits no manifest degradation notice.

## Must keep failing closed

- `includeBuild("../outside") { … }` and every rooted, drive-qualified, UNC, or traversing path —
  refused with a diagnostic naming the path, not the block.
- An interpolated or otherwise dynamic path argument, with or without a block.
- More than one path argument, with or without a block.
- An unbalanced trailing block — refused with an unbalanced-braces diagnostic.
- A trailing expression that is not a configuration block (for example `includeBuild("p") = x`).
- A `}` inside a string literal must not close the block; a `{` inside a string literal must not
  open one; braces inside comments must not move the scan in either direction.
- A block-scoped `include`, a `projectDir` mutation, or an unrecognized `project(...)` mutation
  written **inside** the block fails exactly as it does outside it.

## Must not change

- `includeFlat` and `includeWorkspace` stay token-refused.
- The bare `includeBuild(path)` form, in every spelling and position #723 established.
- Any non-Gradle manifest parser, and any capability-confinement, bounding, or identity behavior.
