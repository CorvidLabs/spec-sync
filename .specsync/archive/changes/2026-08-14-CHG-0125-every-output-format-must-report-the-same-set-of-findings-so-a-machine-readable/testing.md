---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: testing
---

# Testing

`tests/integration/finding_identity_parity.rs` runs ONE broken fixture through
every format of `check` and of `coverage`, extracts the SET of finding
identities from each, and asserts the sets are equal. Presentation may differ;
the set may not.

The fixture used for review was independent of the one the change ships with:
a `pay` module where `charge` is real and documented, `refund` real and
undocumented, and `ghostOnly` documented but nonexistent — so it exercises an
error, a warning, and a coverage figure at once.

Discrimination, personally observed:

    with the fix     12 parity tests pass
    src/ stashed     11 FAIL, 1 passes
    restored         12 pass

Both directions. A clean fixture through all twelve runs is all-clear with exit
0, and `check --format csv` emits `severity,spec,message` and no rows — a
well-formed empty CSV, distinguishable from a run that never happened. A
zero-source tree still reports `no source files to measure` and `null`,
confirming #582 is not regressed.

The staleness half has its own board, because it was the blocker found in
review:

    check --stale 2 --format github   before: exit 1, no warnings section
                                      after:  exit 1, staleness named
    parity loop widened to 4 formats  before: 11 pass / 1 FAIL
                                      after:  12 pass / 0 FAIL

Suite: fmt clean, clippy clean, 2242 unit + 367 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-output-006 | The parity test asserts set equality across twelve runs. Its value is the widened staleness loop: the original looped over `["table","csv"]` — exactly the two arms already fixed — so it could only confirm what had been done, and passed while markdown and github were still wrong |
| REQ-cmd-check-011 | csv rows carry the identities; a comma inside a dependency message renders as one quoted row, not two. Staleness reaches all four non-text formats, proven by the before/after board above |
| REQ-cmd-coverage-003 | `coverage --format json` on the broken fixture carries `passed:false` with errors and warnings, where it previously carried `{file_coverage:100, modules:[], uncovered_files:[]}` |
| REQ-mcp-006 | The two MCP payloads are byte-identical on the broken fixture. Before, they disagreed even on key names — `file_coverage` versus `file_coverage_percent` — which is what three hand-built copies produces |
| REQ-cmd-init-006 | One `csv_field`; both duplicates deleted |
| REQ-cmd-init-registry-002 | The registry initialiser calls the shared implementation |
