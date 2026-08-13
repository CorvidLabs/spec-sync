---
change: CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc
artifact: docs
---

# Docs

## CHANGELOG

`CHANGELOG.md` gains three `Fixed` entries under the unreleased 6.0.0 heading:

- `specsync init` now records the protected SDD files it creates, so the first
  `specsync check` after initialization no longer reports initialization's own output as
  uncovered meaningful delivery.
- Generated spec sections no longer fail the effective-contract gate. A section an
  active change authored and then emptied still fails.
- A directory listed in a spec's `files:` block is now a reported error naming the source
  files to list instead of passing validation with zero exports extracted (#472). The
  snapshot validation path reports it as a directory rather than as an out-of-root
  security escape.

## New public API

Both symbols are documented in their canonical specs as part of this change, because
adding a public function without documenting it is precisely the drift this tool exists
to catch — `specsync check --strict` flagged both before this change was authored.

| Symbol | Spec |
|---|---|
| `change::record_bootstrap_paths` | `specs/change/change.spec.md` |
| `generator::find_module_source_files` | `specs/generator/generator.spec.md` |

## `.specsync/bootstrap.json`

A new on-disk artifact written by `specsync init`. It records the protected SDD paths
that initialization created, the commit it was created against, and a digest of each.
It is not user-authored and should be committed alongside the files it describes.

Editing any file it names revokes that file's exemption, so the record cannot be used to
keep policy changes out of the change workflow.

## No migration owed

6.0 is untagged. A repository initialized by an earlier 6.0 build has no bootstrap
record and is unaffected: the exemption only ever removes a finding.

## Release-note scope

`v6.0.0-rc.1` promises the **product surface** — `check`, `coverage`, drift detection,
export extraction. The `change *` lifecycle verbs are mid-reduction and are explicitly
not part of the RC promise. These three fixes are all on the product surface.
