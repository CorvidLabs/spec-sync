---
change: CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness
artifact: design
---

# Design

## One predicate, five consumers

The defect recurred five times because five sites each asked "can I measure this
file?" in their own words. Three answered with `.exists()`, one added a directory
check, one asked nothing at all. Fixing them individually would leave the sixth
site to be written later against the same wrong intuition.

`source_was_deleted(root, since, path)` answers the question once:

    git cat-file -e <since>:./<path>

The `<rev>:./<path>` form resolves relative to the working directory rather than
the repository root, so a project that sits in a subdirectory of its repository is
answered correctly — the repo-root-relative form would report every file as never
tracked there, quietly and catastrophically.

This mirrors `ExportScan` from #573: classify at the boundary, match exhaustively
at each consumer, so the compiler carries the obligation instead of the author's
memory.

## Deleted is worse than stale, not softer

The first attempt classified a deletion as "unmeasurable". That was wrong, and
git says so: it returns a commit count and names the deleting commit. Calling it
unmeasurable discards the stronger claim and puts `stale` in disagreement with
`lifecycle`, which could already see the deletion.

It is also why the deletion must escape the threshold. It measures one commit;
a default threshold of five swallows it. Reporting it as ordinary drift buries it
exactly as effectively as skipping it did.

## Withheld, not penalised

In `scoring` the temptation was to charge for the deletion in the git dimension.
That double-bills: the file-existence criterion already charges for it, and a
second penalty would move every affected spec's score — a contract change nobody
asked for. The git half's error was asserting a measurement it never made, so it
reports withheld and the score stays where it was.

## The exit code is half the bug

The report was "says everything is fine AND exits 0". Fixing only the message
leaves a gate that still passes, which is the half a CI job reads. The unmeasured
cases now exit non-zero, matching how the same command already treats missing
history.
