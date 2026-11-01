---
change: let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the
artifact: docs
---

# Docs

## `site/src/content/docs/configuration.md`

In the SDD policy section, after the `ignored_paths` example, add a subsection,
"Keeping another tool's records out of verification". It contains:

- The key, with the example `"verification_ignored_paths": [".chough/"]`, and the fact that the
  default is empty.
- What it does: files under a listed directory, tracked or untracked, do not change the
  verification project-input digest, so writing them, or committing them before the review, does
  not stale verification or review.
- What still stales the review: any commit after `change review`, including one that touches only
  a listed directory. Keep such records uncommitted until after `change finalize`, as the
  "review and ship are one step" rule already says.
- What it does not do:
  - It does not change coverage. That is `ignored_paths`.
  - It does not change spec scanning. That is `config.toml` `exclude_dirs` and `exclude_patterns`.
  - It does not change what `change check --commit` stages.
- The rules, each with the refusal it causes:
  - directory prefixes ending in `/`, with no globs, `..`, absolute paths or symlinks;
  - at most 8 entries of at most 128 bytes;
  - no overlap with `.specsync/`, the specs tree, `source_dirs`, any `meaningful_paths` scope,
    files listed in a spec, or policy, principles or custom-artifact files. Listing a directory
    in `ignored_paths` does not change this: `tests/fixtures/` stays refused;
  - entries are relative to the project root, like every other path in `sdd.json`;
  - no overlap with a change's declared `--path`.
- The limit, in plain words: SpecSync cannot tell whether your tests or CI read files under these
  directories. If they do, editing those files leaves verification current without your tests
  running again. Only list directories your build and tests never read.
- Where an invalid entry shows up: every `change` verb that loads policy refuses it, `change status`
  names it as the next action, and `change ship-status` lists it as a blocker.
- Compatibility: SpecSync versions before this release ignore the key and treat evidence recorded
  with it as stale. Raise the SpecSync version your CI pins before adding an entry.
- Editing the list is an edit to `.specsync/sdd.json`. It needs a covering change, and it stales
  existing verification.

## `site/src/content/docs/workflow.md`

In the paragraph that explains why verification goes stale, add one sentence:

> Files under a `verification_ignored_paths` directory are the exception: `change status` and
> `change check` name them under "Verification ignores".

## Elsewhere

- The canonical requirement and Contract text arrive through the approved deltas
  (`deltas/change.md`, `deltas/cmd_change.md`).
- After verification, `specs/change/change.3md` records the lesson on a later plane. The digest
  had no policy input, and the one knob users reached for (`ignored_paths`) means coverage.
- `CHANGELOG.md` is written by the release change. The PR body states the new key and JSON fields
  so the release notes can carry them.
- Chough's own docs (CorvidLabs/chough#12) can point to the key once it is released. That is not
  part of this change.
