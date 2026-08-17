---
change: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-exports-010 | `ExportScan::Directory` is returned by `scan_exported_symbols_full` before `read_to_string` is attempted, and is matched explicitly at every consumer, so the compiler rejects a consumer that fails to handle it — which is what makes "never collapses into Unreadable" structural rather than conventional. The shared-predicate criterion is met by `files_entry_is_directory`, called by validator, score, diff and the export scan rather than reimplemented per command |
| REQ-scoring-005 | `directory_in_files_scores_zero_and_names_directory` fails on an `origin/main` binary (scoring 80 or 75 with no mention of a directory) and passes on the fixed one, asserting total 0, grade F, and the word directory in the criteria. `real_source_file_still_scores_at_or_above_strict_bar` passes on BOTH binaries and is the vacuity control: without it, scoring everything zero satisfies the first test. That `score` stays a metric is covered by the same tests still rendering `--explain` output; `check` being unchanged is covered by the unchanged validator tests |

| REQ-validator-042 | The directory refusal now reaches `files_entry_is_directory` rather than an inline test; unchanged validator tests cover that the refusal text and exit status are the same |
| REQ-cmd-diff-004 | `Directory` joins `Unreadable` in the inconclusive arm, so a directory is listed as inconclusive and never contributes an empty export set |
| REQ-cmd-issues-003 | A directory reached through cap-std metadata maps to the directory snapshot variant, so `check` and confined MCP validation share one answer |
| REQ-cmd-score-002 | The floor comparison is unchanged and still inclusive; the gate closes because `score_spec` returns 0, which the two scoring tests assert directly |
| REQ-cmd-lifecycle-003 | Same inclusive comparison, same basis: a directory scores 0 and fails any positive minimum, matching `check` |
| REQ-cli-args-014 | Help text states the implied inclusive eighty and the directory consequence at the point of use |
| REQ-mcp-007 | The duplicate band table is deleted and `scoring::letter_grade` is called, so one total cannot carry two grades; the removal is 11 lines |

## Measured, before and after

    score --strict, files: names a directory   80/100 [B] exit 0  ->  0/100 [F] exit 1, names "directory"
    check, same spec                           exit 1 "is a directory"  ->  unchanged
    score --strict, files: names a real file   100 exit 0  ->  100 exit 0   (vacuity control)

## Suite

    cargo test                    rc=0   2289 unit, 400 integration, 0 failed
    cargo clippy -- -D warnings   rc=0
    cargo fmt --check             rc=0

## Scope note

Nine source files across nine declared spec modules. That breadth is the fix, not scope creep:
the classification is made once and consumed everywhere the question is asked, which is what
stops `check` and `score` disagreeing about the same path. Seven of the nine modules gain no
requirement — they consume the classification without changing their own contract.

## Whole board

Expected: exactly one gate changes state, 059 FAIL to PASS. No pin drill covers #573 —
confirmed by reading the drills rather than grepping the issue number, after that method missed
drill 034 for #529 and drill 037 for #533 earlier in this release.

## Whole board

All 55 drills, run against this change's release binary (`specsync 6.0.0`):

```
pass=47  fail=8  skip=0  total=55
PASS drills/059-score-strict-directory.sh (1s)
```

The board was `46/9` before this change. Exactly one drill changed state — 059,
the gate this change closes. The eight remaining reds are the known open gates
(049 #540, 050 #536, 052 #537, 053 #534, 054 #433, 056 #426, 057 #416,
060 #577), all unchanged.

Drill 059 is a gate drill (>= 044) and self-flips; no inversion commit is needed.
