# Context

CHG-0150 deleted `validate`'s archive-binding block from `release.yml` (#635).
`.github/scripts/test-validate-release-candidate.py` reads that workflow as **text** and
anchors on a line inside the deleted block:

    start = release.index('          export CHECKS_FILE="$checks_file"')

With the block gone, `index` raises and the whole suite aborts:

    ERROR: test_release_reconstruction_requires_actual_pull_request_event
    ValueError: substring not found
    FAILED (errors=1)

That took down `Classify changed paths`, which every other CI job depends on, so `test`, `fmt`,
`audit`, `coverage`, `spec-check` and the rest all reported `skipping` and both gates failed.

## Why this is a separate change

CHG-0150 declared `affected_paths: .github/workflows/release.yml` at its interview. The test
file is outside that, and delivery scope cannot be widened after the interview (#542). The tool
refused the commit and named this remedy itself:

    error: meaningful changed paths are not covered by an active change:
      .github/scripts/test-validate-release-candidate.py
      cover them: specsync change new "<summary>" … --path …

The blast radius was only visible after CI ran — which is after the point where scope froze.
This is the second instance of that pattern in the same change; the first is recorded on #542.

## What the deleted test asserted

One property of the removed reconstruction: that the workflow run behind the binding check was
reached via a `pull_request` event and never `pull_request_target`. A real property, and a
sensible thing to pin — but its subject no longer exists. Its input, the `SpecSync archive
binding` check run, has had no producer since #499 deleted `post-merge-archive.yml`, which is
the whole of #635.
