---
change: CHG-0131-dependency-analysis-must-not-report-a-clean-graph-built-from-imports-it-could-no
artifact: testing
---

# Testing

Sandbox gate 064 is the judge, and the whole-board check is what separates this
from the attempt it replaces — which also closed its own gate:

    gate 064     before  FAIL (pending)      after  pass=5 fail=0 pending=0
    full board   before  pass=40 fail=15     after  pass=41 fail=14

Exactly one drill changed state.

The case the previous attempt failed on, measured directly:

    src/kt/Core.kt      package com.example.core      (directory does NOT match)
    src/kt/Feature.kt   package com.example.feature; import com.example.core.Core

    -> specs/feature: source imports 'core' but it is not in depends_on   rc=1

Under the previous code this tree produced zero edges and rc=0: layout owners
were non-empty so the empty-owners fallback was skipped, no directory suffix
matched, and the unresolved import was dropped by `filter_map`.

Disclosure, and that it is advisory:

    ⊘ 1 import(s) could not be mapped to a spec module, so they were not
      checked against depends_on: feature imports com.example.missing

with matching JSON, markdown, and diagram-mode stderr — and exit code unchanged.

Noise removed: a pure-Rust project listing `src/ci.yml` and `src/tool.sh` now
prints no disclosure and gets the unqualified success sentence.

Suite: fmt clean, clippy clean, 2275 unit + 371 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-deps-003 | The directory-does-not-match-package fixture now produces an edge where it previously produced silence. A third-party import stays Foreign and silent, an unowned project import is Unattributed and reported, and an ambiguous package is disclosed rather than guessed — three outcomes where there was one `Option` being drained |
| REQ-cmd-deps-003 | The disclosure renders in every format from one site, and the exit code is unchanged by it — checked deliberately, so the disclosure cannot become a gate by accident. The import-concept predicate is an exhaustive match, so a new language forces a decision instead of defaulting to silence |
