---
id: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
state: implementing
type: bug_fix
base_commit: 48e9da28ac45d3bd1d3a759e6142bb3812f3d53c
---

# A configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token

## Intent

a configured source_dirs must survive a manifest discovery failure, and an in-repo includeBuild must be judged by its path rather than its token

## Affected Canonical Specs

- `manifest`
- `validator`
- `types`
- `config`
- `output`
- `cmd_check`
- `comment`
- `cli`
- `generator`

## Acceptance Criteria

- a project whose Gradle settings cannot be parsed but whose source_dirs is explicitly configured runs check and coverage to completion and reports real numbers; the unparseable manifest is disclosed as a coverage notice beside those numbers, in text, markdown, and JSON, rather than replacing them; that notice never gates, because unlike a shrunken denominator it cannot inflate a percentage; degrading reads no byte outside the project root, generates nothing out of the rejected discovery, and leaves the outside tree untouched; a project that did NOT configure source_dirs still fails closed, because its source list came from the discovery that failed; an in-repo includeBuild("vendor/podo-shared") parses, contributes no module, and leaves the root build's include list untouched, wherever the declaration appears; includeBuild("../outside") and dynamic, interpolated, multi-argument, or block-suffixed includeBuild arguments are still refused, now naming the argument rather than the token

## No-spec Rationale

Not applicable
