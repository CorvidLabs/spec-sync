---
change: let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived
artifact: design
---

# Design

## A. Declaration — `[modules."<name>"] owns`

- `owns = ["Tests/AlgorandTests", "Package.swift", "Sources/Algorand/Algorand.docc"]` under the existing `[modules."<name>"]` table; `ModuleDefinition` gains `owns: Vec<String>` beside `files` and `depends_on`. Parsed by `parse_toml_modules_nested`, typed as a string array by the checked parser, written by `config_to_toml`; a module carrying only `owns` still round-trips. Omitted, everything behaves as today.
- An entry is a project-relative file, or a directory that owns everything beneath it, matched by `path_matches_scope` — the vocabulary `affected_paths` already uses. Not globs: `exclude_patterns` globbing lives in the coverage scanner, and acceptance evidence should not acquire a second matching semantics that a manifest reader then has to reproduce.
- Reach: `acceptance_input_owners` consults `owns` for the modules the change DECLARES, ahead of the reserved exact classes. An owned test, fixture, or delivery path is signed under the module instead of `@exact:test` / `@exact:delivery`; a path no declared module owns keeps its exact class; a spec's `files:` list still does not lift a mapped test out of `@exact:test`. Nothing else reads the key: it is not a source mapping, so `specsync check`, coverage, and `find_files_for_module` ignore it and demand no spec coverage for an owned path.
- `.specsync/` and the protected SDD paths are never configurable ownership (`ownership_is_configurable`): the sequence-ledger succession rule reads their exact owners, and the lifecycle's own ledger must not be re-homed by the project it governs.
- A directory entry (`kind: non_file`) is treated exactly like a file: it takes configured ownership, and it needs succession only when it changes.

## B. Succession against history

- Predecessor manifests are immutable and still say `@exact:test`; nothing rewrites them. A reserved exact owner is not a module's claim — it records that no module owned the input when it was signed — so it is retired by whichever module owns the path NOW. `validate_supersedes_semantics` (draft `supersede` and acceptance) accepts a module for an entry whose signed owners are all exact when `module_currently_owns_path` says so, by the same `acceptance_input_owners` rule that will sign the successor's own entry; it refuses otherwise, naming the frozen label and the `owns` remedy. An entry a module signed keeps the historical rule: a module that is not among its signed owners is still refused, so a signed owner's claim is never retired by configuration.
- The succession tuple is unchanged: `(predecessor_id, path, module, predecessor_entry_digest, successor_entry_digest)` under the same domain, with `module` the successor's module and the digest-matches-base-tree rule intact. The successor's signed manifest carrying the path under that module is what proves the module owned the path when it superseded it.
- The walk reads the claimants of a changed exact-only entry from the successors' declared obligations (`succession_claimants`) instead of from the entry — the entry names nobody — and judges each claimant with `successor_covers_input`, the same checks a signed owner's successor passes (declared obligation, authenticated evidence, resolved manifest, matching tuple, manifest carries the successor entry under the module, non-removed semantic item, tuple holds, recursive freshness). One authenticated claimant covers the entry; otherwise every refused claimant is named with its reason, with the frozen label as the input's owner. A signed module owner's entry keeps per-owner coverage: every historical owner still needs its own successor.
- No extra audit record for `@exact:test` → module succession: the two signed manifests and the tuple between them already prove who owned the path when, and a third ledger would only be one more thing to authenticate. `correct-owner` is not involved.
- The exact-only diagnostic keeps the audited-reopen remediation and, wherever the configuration can grant the path, names the supersede alternative — the moment the tool knows the remedy is the moment it is cheapest to say.
