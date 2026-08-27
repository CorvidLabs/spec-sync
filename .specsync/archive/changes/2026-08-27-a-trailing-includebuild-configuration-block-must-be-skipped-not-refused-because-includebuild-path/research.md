---
change: a-trailing-includebuild-configuration-block-must-be-skipped-not-refused-because-includebuild-path
artifact: research
---

# Research

## What #723 actually left behind

`gradle_include_build_target` reads the path argument and confines it, then hands the text after
the closing parenthesis to `require_gradle_include_build_statement_end`, which accepts only
whitespace, `;`, and `}`. That `}` allowance exists so `if (x) { includeBuild("p") }` reaches the
same verdict as the three-line spelling — a fix #723 made after the two disagreed. A `{` was never
allowed, so the trailing block was refused as an unmodelled expression.

The refusal was not incidental: #723's refusal fixture list contains
`includeBuild("vendor/shared") { dependencySubstitution {} }` with the trailing-expression message.
That fixture is what this change flips.

## Reproduced

The published behavior was reproduced locally against the pre-change release binary on a real
Gradle fixture (`settings.gradle.kts` with the reported form plus `include(":app")`):

```
coverage exit=1
{"error": "Coverage inconclusive: Cannot parse Gradle settings manifest settings.gradle.kts:
 Unsupported trailing Gradle includeBuild declaration expression", "inconclusive": true, ...}
```

After the change, the same fixture: `exit=0`, `manifest_notices: []`, `modules: [app, Main]`,
`uncovered_files: [app/src/main/kotlin/Main.kt]` — and nothing under `vendor/shared`, confirming the
included build contributes no source directory.

## Guards the block text still passes through

Read to confirm the skip cannot hide anything (each verified by a test, not by reading alone):

- `reject_non_leading_gradle_includes` advances `search_start` from the declaration token, not from
  the end of the skipped block, so `include` / `include*`-prefixed tokens inside the block are still
  found. A block-scoped `include` hits the existing `gradle_brace_depth(...) != 0` check.
- `reject_unsupported_gradle_project_dir_mutations` and
  `reject_unrecognized_gradle_project_mutations` run over the whole file afterwards, from
  `parse_gradle_settings`, and are unaware of the skip.
- `strip_gradle_comments` runs before all of it, so brace-in-comment is already handled; triple
  quoted strings are blanked there too.
- The module extraction loop keys on lines starting with `include` followed by whitespace, `(`, or a
  quote, so `includeBuild` lines were never module sources and still are not.

## Prior art in this file

`gradle_brace_depth`, `gradle_parenthesized`, `gradle_project_chain_statement`, and
`gradle_contains_mutating_assignment` all use the same quote/escape-aware scan idiom. The new helper
follows it rather than inventing a different one, and uses a signed depth like `gradle_brace_depth`
so the counter cannot wrap.
