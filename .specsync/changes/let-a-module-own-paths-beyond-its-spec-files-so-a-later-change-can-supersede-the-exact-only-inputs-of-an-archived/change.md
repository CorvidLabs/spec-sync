---
id: let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived
state: draft
type: feature
base_commit: 404fe4d6fcef380d3675bab5cc1d2d4786d0401c
---

# Let a module own paths beyond its spec files so a later change can supersede the exact-only inputs of an archived bootstrap change

## Intent

Let a module own paths beyond its spec files so a later change can supersede the exact-only inputs of an archived bootstrap change

## Affected Canonical Specs

- `change`
- `config`
- `types`

## Acceptance Criteria

- `[modules."<name>"] owns` in `.specsync/config.toml` gives a declared module paths beyond its spec's `files:` — a file, or a directory that owns everything beneath it — and an acceptance manifest signs a matching path (a directory entry included) under the module instead of `@exact:test` or `@exact:delivery`; the key is not a source mapping, `specsync check` demands no spec coverage for it, and nothing under `.specsync/` can be owned
- `change supersede --spec <module>` accepts a predecessor entry whose signed owners are all reserved exact labels when the module owns the path under the current configuration, refuses it otherwise naming the frozen label and the `owns` remedy without persisting anything, and still refuses a module that is not a signed owner of an entry a module signed
- The successor walk covers a changed exact-only predecessor entry through any authenticated successor that declared it, judged by every check a signed owner's successor passes; the succession tuple is unchanged (successor module, predecessor entry digest, successor entry digest, digest-matches-base-tree) and no reopen, owner correction, or additional audit record is needed; a workflow-v2 successor that edits, deletes, and re-signs such inputs finalizes, and the bootstrap is successor-covered on `check_project` and `audit_project` before and after the archive commit; the exact-only diagnostic names the supersede alternative beside the audited reopen wherever the configuration can grant the path
- Regression tests: `configured_module_ownership_lets_a_v2_successor_supersede_exact_only_inputs_of_a_bootstrap_change` is refused on 404fe4d6 and passes with the feature; `supersede_refuses_an_exact_only_input_the_configuration_grants_no_module` and `configured_ownership_overrides_reserved_exact_classes_for_declared_modules_only` hold; `fledge run lint` and `fledge lanes run verify` pass

## No-spec Rationale

Not applicable
