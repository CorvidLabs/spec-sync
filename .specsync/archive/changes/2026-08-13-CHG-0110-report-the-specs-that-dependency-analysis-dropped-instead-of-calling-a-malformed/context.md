---
change: CHG-0110-report-the-specs-that-dependency-analysis-dropped-instead-of-calling-a-malformed
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#550, verified by hand against product main before being acted on.

Two commands, one tree, opposite verdicts — and the wrong one came from the command
specifically about dependencies:

```
$ specsync check
  ✗ Frontmatter field `depends_on` must be a YAML list, got a mapping (offending line: `depends_on: {oops`)
exit 1

$ specsync deps
  Modules: 1  Edges: 0
  ✓ All dependency declarations are valid.
exit 0
```

## Root cause

`parse_frontmatter` returns the parsed spec **together with its errors**. `deps` took the
spec and discarded the errors, so a malformed `depends_on` became an empty list: no edges,
no complaints, and an affirmative declaration of validity over frontmatter the validator
rejects outright.

This is the fourth instance of one bug wearing different clothes — success inferred from
the absence of evidence. Draft specs, cold caches, skipped symlinks, unparseable
frontmatter, and now a discarded parse error all produced no findings for want of *input*
and rendered that as a green line.

## Found while fixing it

Two further silent drops in the same walk, both `continue` with no record:

- a spec whose frontmatter cannot be parsed at all
- a spec declaring no `module`

Either one removes a node from the graph without a word, so a project could carry an
unparseable spec indefinitely with `deps` green while edges are missing from both the graph
and the computed build order — which is the output people actually act on.

## Why it matters more than the exit code suggests

`deps` is the command a user runs to ask specifically about dependency health. It is the
last place that should paper over a dependency field it could not read.
