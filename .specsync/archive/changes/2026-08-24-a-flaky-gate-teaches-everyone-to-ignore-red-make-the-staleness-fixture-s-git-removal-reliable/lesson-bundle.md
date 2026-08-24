# Lesson bundle — a-flaky-gate-teaches-everyone-to-ignore-red-make-the-staleness-fixture-s-git-removal-reliable

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A flaky gate teaches everyone to ignore red: make the staleness fixture's git removal reliable
- **Kind**: BugFix
- **Paths**: tests/integration/staleness_unmeasurable.rs
- **Acceptance**: the staleness_unmeasurable module no longer fails intermittently on CI at the .git removal
- **Acceptance**: git background housekeeping is disabled in the fixture so the known concurrent writers are gone
- **Acceptance**: the removal retries on a transient failure and still panics loudly if .git genuinely cannot be removed
- **Acceptance**: no production source or spec text changes

## Evidence

- Verification commit: `5b896a6ea491c9822457f3bb673ece80001008b4`
- Base commit: `f9d034fe1630023e082ed0088ac08862248379e0`
- Verified by: `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

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

## From the change's testing.md

# Testing

`cargo test --release --test integration staleness_unmeasurable` — 12 passed, run three times.

Honest limitation: these tests **always passed locally**, including before this change, so local
runs cannot demonstrate the fix. The flake is CI-specific — Linux, slower disk, higher
parallelism. The proof is CI staying green across subsequent runs, and this change is written so
that if the flake recurs, the retry has already excluded "transient concurrent writer" and the
next investigator starts from a smaller hypothesis space.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
