---
change: drop-windows-from-the-release-qualification-lane-and-the-release-validator-and-state-that-the-retained-windows-content
artifact: context
---

# Context

## What led here

The owner asked to drop Windows CI, after seeing that PR #734 was *adding* a Windows compile job to
ordinary CI and asking the reasonable question: didn't we already remove Windows?

We removed the **binary** (#722). We deliberately kept the **qualification lane**, and #734 was
about giving that lane earlier signal. The owner's decision reverses the retained half.

## The argument being overruled, stated because it was correct

The `### Removed` entry for #722 says the qualification lane must stay:

> It is the only place the retained `#[cfg(windows)]` code is compiled and run, and removing it
> would recreate exactly the condition that produced the `view` defect.

That is true and it is not softened here. `#[cfg(windows)]` code is now compiled and run **nowhere**.
CRLF frontmatter tolerance, reserved-name and Windows-invalid filename guards, `MAX_SLUG_BYTES` and
its `MAX_PATH` justification, junction and reparse-point rejection and path-separator handling are
all retained, still believed correct, and **unverified**. The CHANGELOG paragraph that made the
argument now says the argument was correct and the risk is accepted rather than resolved, instead of
being quietly deleted — a decision reversed silently reads later as a decision never made.

## The case for the reversal

The lane only ever ran on a tag push, so its first signal arrived at the worst possible moment. It
cost `rc.8` and `rc.9`, on a defect latent since #544 that no ubuntu-only job could see. That is the
same argument the Windows binary was dropped on — an artefact nothing exercises — applied one level
further.

## What was kept from #734, and why

The `open_specs_directory` gate fix. It is `#[cfg(test)]` rather than `#[cfg(all(test, unix))]` and
the three helpers are imported unconditionally. **This is correct independently of which platforms
CI compiles for**: a `files:` entry resolving to a directory is a spec-content error on every
platform a repository may be checked out on, and its ambient-path twin
(`validator::tests::directory_source_mapping_fails_loud_and_names_the_files_to_list`) has always
been ungated. Keeping it costs nothing and removing it would re-narrow a platform-independent
guarantee to whichever platform happens to compile it.

What was **dropped** from #734 is the new `windows-check` job in `ci.yml` — the whole point of which
was to protect the lane this change removes.

## The coupling that will bite whoever reverses this

`REQUIRED_PLATFORMS` in the validator and the `qualify` matrix in `release.yml` must move together.
Adding a platform to one without the other fails **every** candidate: the validator demands evidence
the matrix never produces. A comment at the constant says so, and the CHANGELOG entry says so.

## Ruled out

- **Deleting the retained `#[cfg(windows)]` code.** `cargo install specsync` on Windows is still a
  documented path in `README.md`, and mixed-OS teams are the case the content guarantees exist for.
  Unverified is not the same as unwanted.
- **Silently removing the paragraph that argued for the lane.** See above.
- **Keeping a Windows job in ordinary CI while dropping it from the release lane.** That is the
  inverse trade and nobody asked for it: it would pay for Windows signal on every PR to protect a
  lane that no longer exists.
