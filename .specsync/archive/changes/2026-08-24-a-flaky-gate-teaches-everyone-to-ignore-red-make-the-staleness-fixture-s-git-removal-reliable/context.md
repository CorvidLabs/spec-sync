---
change: a-flaky-gate-teaches-everyone-to-ignore-red-make-the-staleness-fixture-s-git-removal-reliable
artifact: context
---

# Context

Three tests in `staleness_unmeasurable` failed on `main` in one run:

    thread '...report_text_and_csv_render_the_absence_rather_than_a_zero' panicked at
    tests/integration/staleness_unmeasurable.rs:88:43:
    Os { code: 39, kind: DirectoryNotEmpty, message: "Directory not empty" }

All three at the same statement — `fs::remove_dir_all(root.join(".git"))` in
`drifted_without_git` — and all in the same parallel run. A fourth failed the same way on a
feature branch earlier and passed on rerun.

`remove_dir_all` reads a directory then unlinks; anything creating a file in between makes it
fail. So something writes into `.git` concurrently on Linux CI.

## What is proven and what is not

The obvious suspect is git's detached housekeeping — `gc --auto` or `maintenance run --auto`
after commit. **That is not proven.** Seven commits sit far below the 6700 loose-object gc
threshold, and a local probe found no leftover gc or maintenance artifacts. The probe could not
have detected a transient lock, so it does not disprove it either, and the flake does not
reproduce on macOS at all.

Rather than ship a confident story about a cause I could not confirm, this does both: disables
the known background writers, and makes the removal itself tolerate a concurrent writer.

## Why a retry is legitimate here

The test's intent is "there is no git history in this tree", not "`.git` can be removed on the
first attempt". A bounded retry serves that assertion. It still panics loudly after ten attempts,
so a directory that genuinely cannot be removed is still a failure rather than a silent pass.

## Why it matters more than a rerun

A flaky gate is worse than a missing one: it teaches everyone to re-run red CI without reading
it. That is the same failure mode as every other defect this release has chased — a signal that
stops carrying information while still looking like one.
