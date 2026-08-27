---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: context
---

# Context

## What led here

Issue #725, a follow-up from the same adopter as #723, reproduced against the published rc.8
binary. Their settings file says:

```kotlin
includeBuild("vendor/podo-shared") {
    dependencySubstitution {
        substitute(module("com.example:podo")).using(project(":"))
    }
}
```

and discovery answers `Unsupported trailing Gradle includeBuild declaration expression`.

#723 taught the parser to judge `includeBuild` by its path argument instead of its token, and
deliberately kept refusing a trailing configuration block. That left the parser accepting the bare
`includeBuild(path)` minority form and refusing the form almost every composite build actually
uses — substituting a local project for a published coordinate is the reason to declare one.

## Why this is not an outage

#723 also fixed the precedence rule that let a discovery failure veto a stated `source_dirs`. That
class-level fix caught this instance nobody anticipated: the adopter now gets a notice naming the
file, the reason, the fallback, and the consequence, and coverage completes on the configured
`source_dirs`. So this is correctness work, not an incident — and nothing here may be traded for
it. Every refusal that protected the path stays exactly as strict.

## Constraints

- An `includeBuild` contributes no module and no source directory, block or no block. The block
  carries substitution rules, not project declarations. That is the entire argument for skipping
  it, and it is why skipping loses no information.
- The path argument is parsed from inside the parentheses, in front of the block. Skipping the
  block must not become a way to carry `../outside`, an interpolation, or a second argument past
  that check.
- The block's text stays in the parsed content. `reject_non_leading_gradle_includes` resumes its
  scan from the declaration token and `reject_unsupported_gradle_project_dir_mutations` runs over
  the whole file afterwards, so a governed directive written inside the block still fails closed.
  This was asserted, not assumed.
- Comments are already stripped before any of this runs, so a brace in a comment is gone. A brace
  in a *string literal* is not, so the brace scan has to be quote- and escape-aware.
- An unbalanced block must fail: its extent is exactly what is unknown.

## Already ruled out

- Accepting a `{` that opens on a later line. The block must open on the declaration's own line, so
  an unrelated following block cannot be mistaken for one. `if (x) { includeBuild("p") }` opens no
  block of its own and keeps its existing treatment.
- Touching `includeFlat` or `includeWorkspace`. `includeFlat` resolves against the parent of the
  root, so its argument is outside the project by construction; `includeWorkspace` is not a form
  this parser models. Neither becomes supportable by reading an argument or skipping a block.
- Supporting the bare Groovy `includeBuild 'path' { … }` spelling, which is not valid Groovy call
  syntax. It keeps failing closed.
